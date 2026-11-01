# spawn：启动可独立调度的任务

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**spawn 把 Future 交给运行时调度，普通 spawn 要求 Future 满足 Send 和 'static。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [Send 与 Sync：哪些值能跨线程](../../concepts/sharing/send-and-sync.md) · [生命周期：引用不能比数据活得久](../../concepts/ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`start_health_checks` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 六个概念分别负责什么

| 概念 | 职责 | 不要混淆 |
|---|---|---|
| `async { ... }` | 创建 Future，描述异步工作 | 创建时不执行块内的工作，也不创建线程 |
| `move` | 让 Future 按值捕获使用到的外部变量 | 不自动 clone，不指定运行线程 |
| `.await` | 推进并等待异步结果 | 等待未就绪时可让出执行机会，并非每次都暂停 |
| `tokio::spawn` | 将 Future 提交为独立调度的任务 | 任务不等于独占一个操作系统线程 |
| `Send + 'static` | 满足 spawn 的跨线程与借用有效性要求 | move 不会自动使 Future 满足这两个要求 |
| 取消与 `Drop` | 任务 Future 被销毁时清理其持有的值 | 丢弃 JoinHandle 不等于销毁正在运行的任务 |

捕获发生在创建 Future 时；块内创建 guard 等工作，要等 Future 被轮询后才执行。

## Arc、Send 与 static 的边界

`tokio::spawn(async move { ... })` 把工作提交给 Runtime，立即返回句柄。
“已经提交”不等于“函数返回时任务已经执行到创建 guard 那一行”。

`async move` 把捕获的值交给 Future，不是“移动到某个指定线程”。
取消测试用 `Arc::clone` 给任务一份所有权，仍然指向同一个 Worker：

```text
测试中的 worker ──Arc──┐
                     ├──> 同一个 Worker / 同一个计数器
任务中的 task_worker ─┘
```

`tokio::spawn` 要求 Future 满足 `Send + 'static`：任务可能在不同线程间移动，也可能比调用方
活得更久。这里 `'static` 不表示任务永远运行；表示不能依赖外部短期借用。
拥有一个 Arc 后，可以在任务内部再借用其中的 Worker。
见 [spawn 的生命周期和 Send 要求](https://tokio.rs/tokio/tutorial/spawning)。

`'static` 不要求任务永远运行，也不要求数据从程序启动就存在。拥有一个 String 的 Future
可以满足 `'static`，只要没有依赖可能失效的外部借用。move 一个短期引用进去，并不会修复它。

`Send` 需要检查 Future 的状态：例如在等待点仍保存的值能否跨线程转移。
如果捕获并跨 await 保存 `Rc`，即使用 async move，也不能提交给要求 Send 的 tokio::spawn。
详细要求见 [Tokio spawning 教程](https://tokio.rs/tokio/tutorial/spawning)。

## 自查与关联

调用 spawn 后是否需要先 await JoinHandle 才开始运行？

[Runtime、任务、线程与 Mio](runtime-task-thread.md) · [JoinHandle：等任务完成，不是任务本体](join-handle.md) · [sleep 与 timeout：等待和时间上限](time.md)
