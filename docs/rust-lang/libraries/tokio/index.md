# Tokio

[总入口](../../index.md) · [问题索引](../../questions.md)

以当前 Cargo.lock 锁定的 **1.53.1** 为例。Manifest 的版本范围和最终锁定版本要分开看。

[对应版本官方文档](https://docs.rs/tokio/1.53.1/tokio/)。按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [select!：等待多个事件](select.md) | 为什么等待停机信号时还要同时等待服务器？落选分支会发生什么？ | [Future](../../concepts/async/future-and-await.md) · [JoinHandle](join-handle.md) | [src/shutdown.rs](../../../../src/shutdown.rs)：`wait_for_shutdown / run` |
| [oneshot：传递一次结果或通知](oneshot.md) | 为什么 send 不用 await，发送成功是否表示处理完成？ | [Move](../../concepts/ownership/move-and-partial-move.md) · [Future](../../concepts/async/future-and-await.md) | [src/shutdown.rs](../../../../src/shutdown.rs)：`shutdown_tx / shutdown_rx` |
| [Runtime、任务、线程与 Mio](runtime-task-thread.md) | Tokio、Mio 与 Netty EventLoop 分别在哪一层？ | [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/main.rs](../../../../src/main.rs)：`#[tokio::main]` |
| [spawn：启动可独立调度的任务](spawn.md) | 调用 spawn 后是否需要先 await JoinHandle 才开始运行？ | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [Send 与 Sync：哪些值能跨线程](../../concepts/sharing/send-and-sync.md) · [生命周期：引用不能比数据活得久](../../concepts/ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`start_health_checks` |
| [JoinHandle：等任务完成，不是任务本体](join-handle.md) | 丢弃 JoinHandle 是否等于取消任务？ | [spawn：启动可独立调度的任务](spawn.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`handle.await` |
| [sleep 与 timeout：等待和时间上限](time.md) | 超时能证明对端没有执行请求吗？ | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`tokio::time::timeout` |
| [取消：abort 之后为什么还要 await](cancellation.md) | abort 返回时，guard 一定已经销毁了吗？ | [JoinHandle：等任务完成，不是任务本体](join-handle.md) · [Drop 与 RAII：离开作用域如何收尾](../../concepts/ownership/drop-and-raii.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/lifecycle.rs](../../../../src/gateway/tests/lifecycle.rs)：`task.cancel_and_join().await` |
| [异步网络：bind、accept 与分段读写](networking.md) | 为什么要累积数据直到读到请求头结束标记？ | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [共享借用与可变借用](../../concepts/ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`start_slow_body_server` |
