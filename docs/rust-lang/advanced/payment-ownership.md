# 扩展：支付对象的所有权选择

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**这是跨领域独立例子；无状态服务不等于没有业务状态，引用和 move 是接口选择。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) |
| 源码入口 | [src/worker.rs](../../../src/worker.rs)：`对照拥有值与借用的差异` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 阅读边界

这是独立支付领域示例，my-smg 没有 Payment 或 Repository 实现。只用于对照所有权选择，不代表当前项目接口。


## 保存是否必须 move

`save(payment, expected_version).await?` 是按值接口的选择，不是保存到数据库必须 move。
若保存过程只是临时读取，可以设计为 `save(&payment, expected_version).await?`，并让接口接收 &Payment。
await 期间外层函数暂停但未退出，局部 payment 仍活着；不能把引用保存到超过其有效期的位置。


## 无状态支付请求中的完整例子

```rust
async fn capture_payment(
    command: CapturePaymentCommand,       // command 所有权 move 进函数
    repository: &PaymentRepository,       // 只借用 repository
) -> Result<PaymentCaptured, CaptureError> {
    let mut payment = repository
        .find(command.payment_id)          // PaymentId 是 Copy
        .await?;                           // 当前函数拥有 Payment

    let expected_version = payment.version();

    let event = payment.capture(
        &command.channel_transaction_id,   // 只借用 String 内容
    )?;

    repository
        .save(payment, expected_version)   // Payment move 给 repository
        .await?;

    Ok(event)                              // event move 给调用方
}
```

这段流程中的所有权变化是：

```text
调用方
  │ move command
  ▼
capture_payment
  │ borrow repository
  │ own + mutate payment
  │ move payment -> save
  │ move event
  ▼
调用方得到 PaymentCaptured
```

这是请求内的内存所有权；不同服务实例之间的支付一致性仍应由幂等键、事务、状态条件和版本号
保证，不能依靠 Rust 的 move 或 borrow。

## Request、Command 和 DTO：一个消费者优先 move

```rust
#[derive(Clone, Debug)]
struct CapturePaymentCommand {
    payment_id: PaymentId,
    idempotency_key: String,
    operator: String,
}
```

如果只有一个消费者，直接转移所有权，不需要 clone：

```rust
async fn handler(command: CapturePaymentCommand) {
    payment_service.capture(command).await;
    // command 已经 move 给 capture
}
```

多个组件确实都需要拥有数据时，再显式 clone：

```rust
async fn handler(command: CapturePaymentCommand) {
    audit_service.record(command.clone()).await;
    payment_service.capture(command).await;
}
```

如果审计只需要幂等键，可以只复制那个字段：

```rust
async fn handler(command: CapturePaymentCommand) {
    let idempotency_key = command.idempotency_key.clone();

    payment_service.capture(command).await;
    audit_service.record_key(idempotency_key).await;
}
```

选择顺序是：

```text
一个消费者  -> move
只是读取    -> &T
多个所有者  -> clone()
进程内共享大型不可变值 -> Arc<T>
```

## 无状态支付服务中的推荐选择

| 类型 | 推荐 | 原因 |
|---|---|---|
| `PaymentId(u128)` | `Copy + Clone` | 小型、固定大小、复制无歧义 |
| `PaymentStatus` | `Copy + Clone` | 小型无资源枚举 |
| 固定大小的 `Money` | 可考虑 `Copy + Clone` | 取决于内部表示 |
| `MerchantOrderNo(String)` | `Clone` | 业务上可复制，但有堆内存成本 |
| HTTP / gRPC DTO | 按需 `Clone` | 单消费者优先 move |
| Command | 按需 `Clone` | 重试或多消费者确实需要时使用 |
| Domain Event | `Clone` 或 `Arc<Event>` | 取决于大小和进程内分发方式 |
| `Payment` Aggregate | 可以 `Clone`，但不应默认滥用 | Clone 只产生内存快照 |
| Query Snapshot / Read Model | 通常可以 `Clone` | 本身就是数据副本 |
| Repository client | 通常可以 `Clone` | 常见实现只是复制连接池句柄 |
| 数据库 transaction | 通常不能 `Clone` | 具有明确事务生命周期 |

最终决策可以简化为：

```text
复制在业务语义上不合理？
  -> 不实现 Clone / Copy

只需要临时读取？
  -> 使用 &T，不复制

值很小、固定大小、没有资源，隐式复制符合直觉？
  -> Copy + Clone

允许复制，但复制可能分配内存或共享底层资源？
  -> Clone，并让调用方显式调用 clone()

需要跨实例保证支付状态正确？
  -> 乐观锁 + 幂等键 + 事务 + Outbox / Inbox
     Clone / Copy 不负责解决这个问题
```

## 独立例子：Payment 的三个接收者

方法接收者直接表达所有权需求：

```rust
struct Payment {
    status: String,
}

impl Payment {
    fn status(&self) -> &str {
        &self.status
    }

    fn capture(&mut self) {
        self.status = "CAPTURED".to_owned();
    }

    fn into_status(self) -> String {
        self.status
    }
}
```

| 接收者 | 含义 | 调用后原值 |
|---|---|---|
| `&self` | 只读借用 | 可以继续使用 |
| `&mut self` | 可变借用 | 借用结束后可以继续使用 |
| `self` | 取得所有权 | 通常不能再使用 |
| `mut self` | 取得所有权，并允许方法内部修改这个绑定 | 通常不能再使用；不是可变借用 |

```rust
let mut payment = Payment {
    status: "AUTHORIZED".to_owned(),
};

println!("{}", payment.status()); // &self
payment.capture();                 // &mut self

let status = payment.into_status(); // self，消费 payment
// payment 此后不能再使用
```

`into_` 开头的方法通常表示取得所有权并转换值，但这是一种命名惯例，不是语法规则。

## 自查与关联

只读保存能否使用 &Payment，为什么？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
