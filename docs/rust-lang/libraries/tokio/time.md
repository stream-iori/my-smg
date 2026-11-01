# sleep 与 timeout：等待和时间上限

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**sleep 让任务等待定时器；timeout 包装 Future 并产生外层超时结果。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`tokio::time::timeout` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## sleep：等待时间，不占住线程

| 操作 | 影响 |
|---|---|
| `std::thread::sleep(...)` | 阻塞当前操作系统线程 |
| `tokio::time::sleep(...).await` | 当前任务等定时器，线程可以推进其他任务 |

`Duration` 是标准库里的时间长度类型；`from_secs(2)` 是 2 秒，`from_millis(1)` 是 1 毫秒。
时间点、时长与单位换算见 [Instant 与 Duration](../../concepts/values/instant-and-duration.md)基础部分。构造时长、计算时间点本身不会等待；这里的 sleep 才负责等待。
异步定时器需要 Runtime 支持。见 [Tokio sleep](https://docs.rs/tokio/1.53.2/tokio/time/fn.sleep.html)。

## timeout：给一段异步等待设上限

本次等待“任务已经开始”的代码：

```rust
let started = tokio::time::timeout(Duration::from_secs(2), async {
    while worker.counter() == 0 {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
})
.await;
```

```text
计数仍为 0 → 等 1 毫秒 → 再检查
计数变为 1 → 退出循环 → started = Ok(())
超过 2 秒  → 停止等待 → started = Err(Elapsed)
```

这个循环不是处理请求，而是确认 spawn 的任务确实创建了 guard。若直接 abort，任务可能
尚未执行，计数一直为 0，那么测试根本没测到 guard 的清理。

超时取消的是被包装的 Future，不会强行中断占住线程不让出的同步代码。
本例包住的是观察循环，**不是另一个已 spawn 的任务**；超时后仍需清理那个任务。
若把 JoinHandle 按值放进 timeout，超时丢弃句柄也不会自动 abort 任务。
见 [Tokio timeout](https://docs.rs/tokio/1.53.2/tokio/time/fn.timeout.html)。

## 自查与关联

超时能证明对端没有执行请求吗？

[Runtime、任务、线程与 Mio](runtime-task-thread.md) · [spawn：启动可独立调度的任务](spawn.md) · [JoinHandle：等任务完成，不是任务本体](join-handle.md)
