# 扩展：支付值对象与 Copy

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**固定大小的值对象可以 Copy，但业务上的实体身份和并发一致性不靠 Copy 决定。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) |
| 源码入口 | [src/circuit_breaker.rs](../../../src/circuit_breaker.rs)：`CircuitState 的 Copy 对照` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 阅读边界

这是独立支付领域示例，my-smg 没有 Payment 或 Repository 实现。只用于对照所有权选择，不代表当前项目接口。



## 无状态支付微服务中的 `Clone` / `Copy`

在 DDD 支付系统中，判断是否实现 `Clone` / `Copy` 时，需要分开看两个问题：

```text
Rust 所有权问题：
当前进程、当前请求中，一个值如何传递和复制？

分布式一致性问题：
不同实例、请求和消息之间，支付状态如何保持正确？
```

`Clone` / `Copy` 只回答第一个问题。第二个问题通常由数据库事务、状态条件、乐观锁、幂等键、
Outbox 和消息去重解决。

## “无状态”不等于没有业务状态

无状态服务不在某个进程中长期保存权威业务状态。一次请求通常经历：

```text
请求到达
  -> 从数据库或消息重建当前状态
  -> 在内存中短暂构造领域对象
  -> 执行业务规则
  -> 条件更新数据库并写入 Outbox
  -> 请求结束，内存对象释放
```

持久状态仍然存在，只是位于进程外：

| 位置 | 典型状态 |
|---|---|
| 数据库 | 支付状态、金额、版本号 |
| Redis | 幂等记录、短期缓存 |
| 消息队列 | Command、Domain Event |
| Outbox | 已提交但尚未发布的事件 |

所以同一笔支付天然可能在不同实例中出现多个内存副本：

```text
实例 A：Payment(id=100, status=Authorized, version=7)
实例 B：Payment(id=100, status=Authorized, version=7)
```

即使 `Payment` 没有实现 `Clone`，两个实例仍能分别从数据库加载出这两个值。因此：

```text
禁止 Clone != 防止分布式并发
```

## `Copy`：适合小型、固定大小的值对象

支付 ID 是典型例子：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PaymentId(u128);

fn load_payment(id: PaymentId) {}
fn record_metric(id: PaymentId) {}

let id = PaymentId(100);
load_payment(id);
record_metric(id); // id 自动 Copy，仍可继续使用
```

复制 `PaymentId` 只是复制身份值，不是创建一笔新支付：

```text
复制 PaymentId(100) -> 得到相同的编号值
创建新 Payment      -> 应由领域流程产生新的唯一 ID
```

固定大小的金额和状态枚举也可以考虑 `Copy`：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Currency {
    Cny,
    Usd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Money {
    minor_units: i64,
    currency: Currency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaymentStatus {
    Created,
    Authorized,
    Captured,
    Refunded,
    Failed,
}
```

这里的判断依据不是“它们属于 DDD Value Object”，而是：

- 复制在业务语义上合理；
- 类型很小且大小固定；
- 所有字段都是 `Copy`；
- 没有堆内存或其他资源所有权。

值对象不一定能 `Copy`。例如订单号包含 `String`，只能按需 `Clone`：

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct MerchantOrderNo(String);
```

```text
DDD 分类回答：这个值在业务上是否允许复制？
Rust 类型回答：它在技术上能否安全地隐式复制？
```

## 自查与关联

小值对象与可变业务实体应如何区分？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
