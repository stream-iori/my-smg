# oneshot：传递一次结果或通知

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**oneshot::channel 创建一对发送端和接收端；最多发送一个值，发送不等待，接收可以异步等待。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Move](../../concepts/ownership/move-and-partial-move.md) · [Future](../../concepts/async/future-and-await.md) · [Option 与 Result](../../concepts/errors/option-and-result.md) |
| 源码入口 | [src/shutdown.rs](../../../../src/shutdown.rs)：`shutdown_tx / shutdown_rx`；[HTTP 生命周期测试](../../../../src/gateway/tests/lifecycle/http.rs)：`fail / peer_closed` |
| 阅读时机 | 阶段 12；依据 Tokio 1.53.1，项目停机代码另行验收。 |

## 从 Java 视角理解两端

```text
let (tx, rx) = oneshot::channel::<T>();
     │   │
     │   └─ Receiver<T>：等待一次结果
     └───── Sender<T>：提交一次结果

生产者拥有 tx ── send(value) ──> 通道保存 value ── rx.await ──> 消费者
```

Java 可类比“一个 CompletableFuture 的一次完成与等待”，但 oneshot 把能力分为两个不能随意复制的端点。这里是一次点对点传递，不是广播，也不是持续发送消息的队列。创建通道不会启动任务。

## 例一：工作任务交回一个结果

```rust
async fn calculate() -> u64 {
    let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

    let worker = tokio::spawn(async move {
        let answer = 6 * 7;
        tx.send(answer).expect("receiver should still exist");
    });

    let answer = rx.await.expect("worker should send an answer");
    worker.await.expect("worker should finish normally");
    answer
}
```

本例为演示两端，若只是拿工作任务的最终返回值，直接 `JoinHandle<u64>` 更简单。oneshot 更适合让已有长期任务回传某个阶段的结果，或者让独立的控制流程给服务器发一次通知。

| 操作 | 返回与所有权 |
|---|---|
| `channel::<T>()` | 得到 `(Sender<T>, Receiver<T>)` |
| `tx.send(value)` | 消费 tx 和 value；返回 `Result<(), T>`，失败时把 value 还给调用方 |
| `rx.await` | 通常按值消费接收端，得到 `Result<T, RecvError>` |
| `(&mut rx).await` | 借用接收端等待；未完成时可取消这次等待并保留端点 |

send 不需要 await，因为只向这个一次性通道提交一个值，不等待对方处理。Receiver 自己实现 Future，因此直接 rx.await，不使用 mpsc 风格的 rx.recv().await。依据：[oneshot 模块](https://docs.rs/tokio/1.53.1/tokio/sync/oneshot/index.html)。

## 例二：一次通知为什么用 ()？

```rust
async fn notify_once() {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // 可以先发送，再等待接收；通道会保存这一个值。
    tx.send(()).expect("receiver is alive");
    assert!(rx.await.is_ok());
}
```

`()` 是 unit 类型，表示不携带业务数据；这里真正有意义的是“通知发生了”。不要把 `send(())` 看成“没有发送任何东西”。

停机代码把 rx 移进 Axum 的 graceful shutdown Future；管理流程检测信号后调用 tx.send(())，使接收等待结束。这个通知只负责开始停机，**不表示已有请求已经全部结束**；完成状态仍需等待服务器句柄。

```text
管理流程                      Axum 服务器任务
等待 Ctrl+C / SIGTERM         等待 rx
tx.send(()) ────────────────> rx.await 完成
                              停止接收，排空连接
等待 server_task <──────────  服务器结束
```

## 失败与 Drop

| 情况 | 另一端观察到什么 |
|---|---|
| tx 正常发送，rx 继续接收 | rx.await 得到 Ok(value) |
| tx 未发送就被销毁 | rx.await 得到 RecvError，不会因为没有消息永久等待 |
| rx 已销毁或关闭接收 | tx.send(value) 返回 Err(value) |
| send 返回 Ok 后，接收方又放弃 | 仍可能没有业务处理；Ok 不是处理完成确认 |

Sender::send 消费 self，防止使用同一个发送端再次发送；tx/rx 不提供可随意复制的多生产者/多消费者语义。若要确认对方处理完毕，需要另设确认通道或等待任务句柄。见 [Sender::send](https://docs.rs/tokio/1.53.1/tokio/sync/oneshot/struct.Sender.html#method.send)。

项目使用 `let _ = shutdown_rx.await`，主动选择“收到通知或发送端消失，都触发停机”。普通业务场景可能需要区分两者，不要到处忽略 RecvError。`let _ = shutdown_tx.send(())` 则表示接收方消失时也继续清理；不代表所有发送错误都应忽略。

## 与 select! 一起使用

```rust
async fn wait_or_timeout() {
    let (tx, mut rx) = tokio::sync::oneshot::channel::<u64>();

    let result = tokio::select! {
        result = &mut rx => Some(result),
        _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => None,
    };

    // tx 还活着且没有发送，接收分支不会完成，定时分支会获胜。
    assert!(result.is_none());

    // 先前只取消了对 &mut rx 的等待，没有销毁 rx。
    tx.send(42).expect("receiver was preserved");
    assert_eq!(rx.await.expect("message should arrive"), 42);
}
```

`&mut Receiver<T>` 的等待具有取消安全性：另一分支获胜时，不会悄悄消费消息；之后还能继续接收。若把 rx 按值移入 select!，落选时接收端就会销毁。接收已经完成后应退出相关循环，不能再次等待已完成的 Receiver。见 [Receiver 取消安全性](https://docs.rs/tokio/1.53.1/tokio/sync/oneshot/struct.Receiver.html#cancel-safety)。

## 自查与关联

1. 为什么 send 没有 await，而接收需要 await？
2. send 返回 Ok，能证明对方业务处理完成了吗？
3. 发送端不发送就销毁，接收端会一直挂住吗？
4. 为什么停机通知后还要等待服务器句柄？

[select!](select.md) · [JoinHandle](join-handle.md) · [取消](cancellation.md)
