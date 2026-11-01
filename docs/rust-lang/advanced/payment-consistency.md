# 扩展：Clone 与持久化一致性

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**内存副本不替代数据库版本约束，Clone 与并发写入一致性不是同一个问题。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) |
| 源码入口 | [src/circuit_breaker/gate.rs](../../../src/circuit_breaker/gate.rs)：`对照 generation 的版本校验` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 阅读边界

这是独立支付领域示例，my-smg 没有 Payment 或 Repository 实现。只用于对照所有权选择，不代表当前项目接口。



## Entity / Aggregate 可以 `Clone`，但它只产生内存快照

无状态微服务中，不能简单规定 Entity 或 Aggregate 永远不能实现 `Clone`。例如为了记录状态转换
前后的差异，可以明确复制一个请求级快照：

```rust
#[derive(Clone, Debug)]
struct Payment {
    id: PaymentId,
    status: PaymentStatus,
    version: u64,
    channel_transaction_id: Option<String>,
}

let payment = repository.load(payment_id).await?;

let before = payment.clone();
let mut after = payment;

let event = after.capture()?;
audit.record_transition(&before, &after).await?;
repository.save_with_version(after, before.version, event).await?;
```

这里的两个值表示：

```text
before -> 状态转换前的内存快照
after  -> 状态转换后的内存快照
```

它们不是两笔新支付，也不代表数据库允许两个状态同时写入。

是否为 Aggregate 派生 `Clone` 仍然要谨慎：自动派生会复制全部字段，容易产生不必要的大对象
复制，也可能让单次请求中的状态分支变得难以追踪。如果只需要查询或审计数据，可以定义专门的
只读快照：

```rust
#[derive(Clone, Debug)]
struct PaymentSnapshot {
    id: PaymentId,
    status: PaymentStatus,
    version: u64,
    channel_transaction_id: Option<String>,
}

impl From<&Payment> for PaymentSnapshot {
    fn from(payment: &Payment) -> Self {
        Self {
            id: payment.id,
            status: payment.status,
            version: payment.version,
            channel_transaction_id: payment.channel_transaction_id.clone(),
        }
    }
}
```

```text
Payment         -> 有行为、负责维护业务不变量的请求级聚合
PaymentSnapshot -> 用于查询、审计或传输的只读数据
```

不为 Aggregate 实现 `Clone` 可以作为防止请求内误用的 API 设计，但不能替代分布式一致性机制。

## 支付一致性依靠版本条件，而不是禁止 `Clone`

假设两个实例同时加载版本 7：

```text
实例 A：准备 Capture
实例 B：准备 Refund
```

持久化时需要使用状态条件或乐观锁：

```sql
UPDATE payments
SET status = 'CAPTURED',
    version = version + 1
WHERE payment_id = ?
  AND status = 'AUTHORIZED'
  AND version = 7;
```

```text
affected_rows == 1 -> 更新成功
affected_rows == 0 -> 状态或版本已变化，拒绝或重新加载后重试
```

典型支付流程是：

```text
检查幂等键
  -> 加载 Payment + version
  -> 执行领域状态机
  -> 使用 version 做条件更新
  -> 同一数据库事务写 Outbox
  -> 消费端使用 Inbox / event_id 去重
```

无论当前请求有没有调用 `.clone()`，最终写入都必须经过这条一致性边界。

## Domain Event：按分发方式选择 `Clone` 或 `Arc`

领域事件是已经发生的不可变事实，常常需要交给多个进程内处理器：

```rust
#[derive(Clone, Debug)]
struct PaymentCaptured {
    payment_id: PaymentId,
    amount: Money,
    channel_transaction_id: String,
}
```

事件较小时可以 clone：

```rust
message_handler.handle(event.clone()).await?;
audit_handler.handle(event).await?;
```

进程内广播大型事件时，可以共享同一个不可变值：

```rust
use std::sync::Arc;

let event = Arc::new(event);

handler_a.handle(Arc::clone(&event)).await?;
handler_b.handle(Arc::clone(&event)).await?;
```

`Arc` 只解决当前进程的所有权共享。事件一旦通过 Kafka 等消息系统跨进程发送，仍然要经过
序列化、持久化和消费者去重，不能依靠 `Arc`。

## Repository client 与数据库 transaction 不一样

Repository client 内部如果只是连接池句柄，通常可以廉价 Clone：

```text
PaymentRepository
  -> clone 连接池句柄
  -> 多个请求共享底层连接池
```

但数据库 transaction 有明确的 commit / rollback 生命周期，通常不应实现 `Clone`：

```rust
async fn save_payment(
    tx: &mut PaymentTransaction,
    payment: &Payment,
) -> Result<(), RepositoryError> {
    // 使用同一个事务执行更新和写 Outbox
}
```

如果 transaction 可以随意复制，就会产生“谁提交、谁回滚、是否仍是同一事务”等语义歧义。

## 自查与关联

禁止 Clone 是否能替代乐观锁？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
