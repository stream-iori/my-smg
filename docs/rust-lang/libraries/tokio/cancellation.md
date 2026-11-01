# 取消：abort 之后为什么还要 await

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**abort 请求取消，等待句柄完成才能确认任务资源释放；Drop 无法异步等待。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [JoinHandle：等任务完成，不是任务本体](join-handle.md) · [Drop 与 RAII：离开作用域如何收尾](../../concepts/ownership/drop-and-raii.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/lifecycle.rs](../../../../src/gateway/tests/lifecycle.rs)：`task.cancel_and_join().await` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 结束、取消与释放

| 情况 | 持有的数据如何清理 |
|---|---|
| 正常执行离开局部作用域 | 相应局部值按作用域规则释放 |
| 未完成的 Future 被销毁 | 已捕获和已创建且仍保存的值被释放 |
| 普通异步任务收到 abort | 调度器完成取消后，销毁任务 Future；等待句柄确认清理 |
| 只丢弃 JoinHandle | 任务脱离句柄继续运行，不代表取消 |

尚未被轮询时，块内 guard 还没有创建，自然没有计数要减；已经创建后被取消，
才通过 Drop 清理。返回给调用方或移交给其他任务的值，由新的所有者负责后续释放。
取消不能自动撤销远端已执行的工作。完整过程见 [第 7 节](cancellation.md)。

## abort 与 Drop：取消测试完整过程

```text
测试函数                          被 spawn 的任务
创建 Arc<Worker>
spawn ──────────────────────────> 创建 guard：计数 +1
观察到计数 1                      pending.await：等待
abort ──────────────────────────> 发起取消
等待 JoinHandle                   Future 被销毁，guard Drop：计数 -1
确认 JoinError 是 cancelled
检查计数为 0
```

`abort()` 发起取消，不保证调用返回时任务已经结束；接着等待 `task.await`，才能确认清理完成。
对于正在进行的普通异步任务，取消会销毁它保存的局部值，因此 guard 的 Drop 能减一。
如果任务已正常完成，abort 也可能得到正常结果；本例用 pending 避免正常完成路径。
见 [任务取消规则](https://docs.rs/tokio/1.53.2/tokio/task/index.html)。

取消不是强制杀线程：不让出执行机会的 CPU 循环会拖延取消；已经运行的 `spawn_blocking`
任务也不能用 abort 停掉。取消更不代表数据库写入或远端请求会自动回滚。
见 [JoinHandle::abort](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinHandle.html#method.abort)。

这是本次教学测试的流程说明，不代表当前项目已经实现了全部场景。测试的意义是证明：
**任务没有走到正常结尾，计数仍能通过 Drop 归零。**

## 自查与关联

abort 返回时，guard 一定已经销毁了吗？

[Runtime、任务、线程与 Mio](runtime-task-thread.md) · [spawn：启动可独立调度的任务](spawn.md) · [JoinHandle：等任务完成，不是任务本体](join-handle.md)

停机流程中的区别：[select! 落选会取消什么](select.md#落选意味着取消什么) · [oneshot 通知不等于清理完成](oneshot.md)。
