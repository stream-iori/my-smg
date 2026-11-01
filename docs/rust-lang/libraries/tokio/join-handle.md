# JoinHandle：等任务完成，不是任务本体

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**await 句柄取得任务结果，外层 JoinError 与任务业务错误是两层。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [spawn：启动可独立调度的任务](spawn.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`handle.await` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## JoinHandle：不是 Runnable

名字是 `JoinHandle`，不是 `JoinHandler`。它是已经提交的任务的控制句柄。

| Rust | Java 中用途较接近的概念 |
|---|---|
| 提交的异步工作 | `Runnable` / `Callable` 描述的工作 |
| `tokio::spawn(...)` | `ExecutorService.submit(...)` |
| Tokio `JoinHandle<T>` | 提交后返回的 `Future<T>` |

这是职责类比，不代表运行或取消机制相同；Rust 的 Future 也不等于 Java 的 Future。

| 类型 | 来源 | 等待方法 |
|---|---|---|
| `std::thread::JoinHandle<T>` | 标准库线程 | `.join()`，阻塞调用线程 |
| `tokio::task::JoinHandle<T>` | Tokio 异步任务 | `.await`，得到 `Result<T, JoinError>` |

任务不返回额外数据时，句柄是 `JoinHandle<()>`。正常完成得到 `Ok(())`，取消或 panic
得到 `JoinError`。`is_cancelled()` 用来区分取消。
**丢弃句柄会让任务脱离句柄继续运行，不等于取消。**
参见 [Tokio JoinHandle](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinHandle.html)。

## 自查与关联

丢弃 JoinHandle 是否等于取消任务？

[Runtime、任务、线程与 Mio](runtime-task-thread.md) · [spawn：启动可独立调度的任务](spawn.md) · [sleep 与 timeout：等待和时间上限](time.md)

需要同时等待任务结束与停机通知时，见 [select!](select.md)；一次性通知的传递见 [oneshot](oneshot.md)。

停机流程的详细设计见 [临时借用 JoinHandle，保留后续清理能力](select.md#阶段-12临时借用-joinhandle保留后续清理能力)：借用可以结束，句柄仍保留；结果一旦取出则不能再次 await。
