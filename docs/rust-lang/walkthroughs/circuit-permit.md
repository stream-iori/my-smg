# 代码导读：许可、状态版本与迟到结果

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**凭证保存 Gate 和本轮身份，消费式上报防重复，版本检查拒绝旧结果干扰新状态。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) · [Drop 与 RAII：离开作用域如何收尾](../concepts/ownership/drop-and-raii.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/circuit_breaker/gate.rs](../../../src/circuit_breaker/gate.rs)：`CircuitPermit` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 当前代码的状态与凭证

```text
Arc<CircuitGate>
    └── Mutex<GateState>
         ├── CircuitBreaker
         └── generation：Arc<()>

CircuitPermit
├── gate：Arc<CircuitGate>
├── generation：申请时克隆的身份
└── finished：是否已提交结果
```

acquire 返回 Result<Option<CircuitPermit>, CircuitGateError>：

| 返回 | 含义 |
|---|---|
| Ok(Some(permit)) | 允许本次尝试 |
| Ok(None) | 熔断策略拒绝，不代表锁坏了 |
| Err(Poisoned) | 共享状态不可安全读取，返回明确内部错误 |

success/failure 消费凭证并标记 finished，再尝试更新 Gate；标记 finished 不保证更新成功，失败仍向调用者返回错误。
未提交的 permit 在 Drop 时尝试取消；普通请求取消不计失败，半开探测取消会重新打开熔断，避免探测永久占位。
如果 generation 身份已变化，旧结果被忽略，不能干扰新一轮状态。

## Permit 显式保存请求与熔断器的关系

CircuitPermit 表达的契约是：

> 这一次尝试，被这个熔断器、在这个状态版本下允许执行。

```text
共享 CircuitGate
    └─ Mutex<GateState>
          ├─ CircuitBreaker：共享状态机
          └─ generation：当前状态版本的身份标记

请求 A → Permit A
           ├─ gate：Arc 指向上述同一个 CircuitGate
           ├─ generation：申请时保存的版本标记
           └─ finished：是否已经报告结果

请求 B → Permit B → 同一个 Gate，不是另一份 Breaker
```

| 对象 | 生命周期与职责 |
|---|---|
| CircuitBreaker | 保存状态，决定状态转换 |
| GateState | 把 breaker 和当前版本放在一起，维护一致性 |
| CircuitGate | 提供共享入口，在锁内检查和更新 |
| CircuitPermit | 每次获准尝试一份，关联共享 Gate 和申请时版本 |

计划集成时，每个 Worker 一套共享 Gate；每次重试重新申请凭证。
这是后续集成方式，不代表当前 HTTP 链路已经使用熔断器。

## 为什么需要状态版本：A、B、C 的例子

假设一次失败触发熔断。①②③只是讲解标签，Arc 标记里没有保存这些数字。

| 顺序 | 事件 | 共享状态与版本 | 凭证 |
|---|---|---|---|
| 1 | A、B 在正常状态下获准 | Closed，版本① | A=①，B=① |
| 2 | B 失败，触发熔断 | Open，换成② | A 仍持有① |
| 3 | 冷却后放行探测 C | HalfOpen，换成③ | C=③ |
| 4 | A 的旧成功结果返回 | 仍是 HalfOpen，③ | ①不匹配，忽略 |
| 5 | C 成功返回 | 校验③通过，恢复 Closed 并换版本 | 接受本轮探测结果 |

如果只判断“当前 HalfOpen，收到成功”，就可能把 A 当成探测 C。

```text
请求身份：这是 A、B 还是 C？
状态版本：这个请求在什么状态版本下获准？
```

A、B 共用版本，不是因为同时到达，而是它们申请之间没有状态转换。
之前讨论中的“轮次”容易被误解为请求批次，**状态版本**是更清楚的说法。
当前设计在状态转换时使旧凭证失效，这是一个明确的业务策略，不是所有系统都必须采用的规则。

## Arc<()> 保存什么

它不保存请求 ID、状态值或版本数字。`()` 没有业务数据，Arc 用来共享一个可比较身份的标记。

| 操作 | 含义 |
|---|---|
| `Arc::new(())` | 创建一个新身份标记 |
| `Arc::clone(&marker)` | 共同持有同一个标记，不创建新身份 |
| `Arc::ptr_eq(&a, &b)` | 检查是否同一个标记 |

Java 中可以类比：

```java
Object currentVersion = new Object();
Object requestVersion = currentVersion;

currentVersion = new Object();

boolean stillValid = requestVersion == currentVersion; // false
```

也可以改用数字版本，但必须避免版本溢出后重复使用而误认旧凭证。
必要的是“识别过期结果”，不是 generation 这个名字，也不是 Arc<()> 这一种实现。
详见 [Arc：对象身份与值相等](circuit-permit.md)。

## 当前练习还缺什么

- HTTP 集成：何时申请、每次重试如何记录结果、健康检查如何与熔断区分。
- 毒锁策略：当前 into_inner 只拿回守卫，不证明 breaker 与版本依然一致。
- 取消验收：除直接 drop 外，还需要覆盖实际异步任务取消。

因此不要把这份设计感悟当作“并发集成已经完成”的验收记录。
毒锁问题见 [MutexGuard 与毒锁](../concepts/sharing/mutex-and-guards.md)，
取消过程见 [Tokio：abort 与 Drop](../libraries/tokio/cancellation.md)。

## ptr_eq：对象身份与值相等

`Arc::ptr_eq(&first, &second)` 返回 `bool`，检查两份 Arc 是否指向**同一份分配、同一个对象**，
而不是检查对象内容是否相等。它借用两份 Arc，不移动所有权，也不增加强引用计数。
参见 [标准库 Arc::ptr_eq](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.ptr_eq)。

```rust
use std::sync::Arc;

let original = Arc::new(String::from("hello"));
let cloned = Arc::clone(&original);
let same_value = Arc::new(String::from("hello"));

assert!(Arc::ptr_eq(&original, &cloned));
assert!(!Arc::ptr_eq(&original, &same_value));
assert_eq!(original, same_value); // 内容相同，但不是同一个对象
```

```text
original ──┐
cloned ────┴──> String("hello")，对象 A
same_value ───> String("hello")，对象 B
```

| 检查 | 问的问题 | 需要内部类型实现 PartialEq 吗 |
|---|---|---|
| `Arc::ptr_eq(&a, &b)` | 是不是同一个对象 | 不需要 |
| `a == b` / `assert_eq!(a, b)` | 内部值按 PartialEq 规则是否相等 | 需要 |

在节点注册表的“拒绝重复 ID”测试里，原节点和重复节点可以有相同 ID，但它们是两个对象。
检查 `Arc::ptr_eq(&original, &found)`，才能确认 get 返回的仍是原对象，而不是被偷偷替换成
另一个同名节点。只检查 ID 相等，发现不了这种替换。

## 自查与关联

为什么 Arc<()> 身份比只比较状态枚举更有用？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：配置加载与分层错误](configuration-loading.md)
