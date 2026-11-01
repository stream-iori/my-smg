# 代码导读：轮询选择与策略状态

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**选择结果是快照下标，轮询计数保存在长期存在的策略实例里。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../src/policy.rs)：`RoundRobin::select` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 沿当前 select 看输入输出

```text
Vec<Arc<Worker>> 快照
        ↓ 共享切片
healthy_worker_indices → Vec<usize>（原快照索引）
        ↓
RoundRobin.next % 健康节点数 → 健康索引数组中的位置
        ↓
取出原快照索引 → Some(index)，同时推进 next
```

空健康列表返回 None；不能把健康列表位置直接当成原快照索引。
GatewayState 将策略保存在 Mutex<Box<dyn Policy + Send>> 中，不在每个请求中新建策略，否则 next 总从 0 开始。

当前 Policy::select 接收 &mut self 与 &[Arc<Worker>]；早期同步练习使用 &[Worker]，这是演进前后的签名，不是当前代码矛盾。

## 动态分发与工厂方法

下面只是调用已有策略，不是工厂：

```rust
fn select_with_dynamic_policy(
    policy: &mut dyn Policy,
    workers: &[Worker],
) -> Option<usize>
```

真正的策略工厂会创建并返回策略：

```rust
fn create_policy(name: &str) -> Box<dyn Policy> {
    match name {
        "first-healthy" => Box::new(FirstHealthy::new()),
        "round-robin" => Box::new(RoundRobin::new()),
        _ => panic!("unknown policy"),
    }
}
```

教学项目后续应把未知配置改成明确的 `Result`，这里的重点只是观察返回类型：不同分支的具体
类型不同，但都能被擦除为 `Box<dyn Policy>`。

## `my-smg`：配置驱动的共享策略

阶段 6 的网关根据配置选择 `FirstHealthy` 或 `RoundRobin`。两种具体类型不同，但需要放进同一个
字段，因此使用同一类型的 trait object：

```rust
let policy: Box<dyn Policy + Send> = match &config.policy {
    PolicyKind::FirstHealthy => Box::new(FirstHealthy::new()),
    PolicyKind::RoundRobin => Box::new(RoundRobin::new()),
};
```

| 组成 | 在这里的职责 |
|---|---|
| `Box` | 独占拥有选出的具体策略，给字段一个已知大小的指针类型 |
| `dyn Policy` | 擦除具体类型，通过 `Policy::select` 动态调用 |
| `+ Send` | 保证内部策略可安全地转移到其他线程 |

`Send` 不表示多个线程可以同时修改策略。`Policy::select(&mut self, ...)` 需要独占可变访问，
所以网关把它放在 `Mutex` 中，再把整个应用状态放进 `Arc`：

```text
多个请求 ──Arc──> GatewayState
                       └─ Mutex<Box<dyn Policy + Send>>
                                       └─ RoundRobin { next }
```

`Mutex<T>` 能在线程间共享的关键条件之一是 `T: Send`；这里的 `T` 正是
`Box<dyn Policy + Send>`。锁让每次 `select` 串行修改 `next`，`Arc` 让所有请求访问同一份
`GatewayState`。锁的取得、Guard 的释放见
[`Mutex`、`MutexGuard`、解引用与毒锁](../concepts/sharing/mutex-and-guards.md)。

策略必须在请求之间保留状态。如果在每次 `chat` 请求中重新执行 `RoundRobin::new()`，`next`
每次都从 0 开始，有两个健康节点时会一直选第一个：

```text
共享同一个 RoundRobin：worker-1 → worker-2 → worker-1
每次请求重新创建：       worker-1 → worker-1 → worker-1
```

这里保存的是**轮询进度**。它不是“会话亲和性”：后者通常指同一客户端的请求持续进入同一节点。
`match &config.policy` 为什么借用字段，见[字段借用与 `match` 表达式](../concepts/values/expressions-and-semicolons.md)。

## 机制与策略分离

每个组件只回答一个问题：

| 组件 | 回答的问题 |
|---|---|
| `Display` | 这个错误怎样显示成人类可读文字？ |
| `Error` | 它能否进入标准错误接口，底层原因是谁？ |
| `From` | 下层错误怎样包装成上层错误？ |
| `?` | 失败时怎样快速返回？ |
| 日志层 | 何时、以什么级别记录？ |
| `main` / HTTP 边界 | 打印、退出还是转换成状态码？ |

反例：在 `validate()` 中直接打印错误。

```text
validate 同时负责校验和输出
        │
        ├─ 测试难控制输出
        ├─ 库代码替调用方做决定
        └─ HTTP、CLI、测试无法选择不同处理方式
```

更好的方式是让 `validate()` 返回结构化错误，由最外层决定如何展示。

## 自查与关联

每次新建 RoundRobin 为什么总从第一个开始？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：配置加载与分层错误](configuration-loading.md) · [代码导读：一次聊天请求](chat-forwarding.md)
