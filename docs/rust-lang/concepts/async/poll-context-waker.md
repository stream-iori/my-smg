# Poll、Context 与 Waker：等待和唤醒

[分类目录](index.md) · [Future 与 await](future-and-await.md) · [Pin 与 Unpin](pin-and-unpin.md) · [问题索引](../../questions.md)

**Future 保存一次异步计算的状态；执行器通过 poll 推进它；不能继续时，Waker 让任务在条件变化后有机会再被推进。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Future 与 await](future-and-await.md)；了解所有权和借用 |
| 项目入口 | [写停滞超时](../../../../src/write_timeout_listener.rs)：`check_timeout` 与 `poll_write` |
| 相关导读 | [SSE 调度与所有权](../../walkthroughs/sse-scheduling.md) |

## 谁负责什么

| 名称 | 职责 | 不要误解成 |
|---|---|---|
| Future | 一个可被推进的异步计算值，完成后产生 `Output` | 自动运行的后台线程 |
| `poll` | 尝试推进一次 Future | 阻塞等待结果的函数 |
| `Poll::Ready(value)` | 本 Future 已完成，交出结果 | 一定是业务成功；`value` 本身可以是 `Err` |
| `Poll::Pending` | 这次无法完成 | 已完成或一定在固定时间后重试 |
| `Context` / `cx` | 本次 poll 的任务上下文，稳定接口提供当前任务的 `Waker` | HTTP 请求、业务状态的容器 |
| `Waker` | 通知执行器对应任务应当再次获得 poll 的机会 | 直接在当前线程执行 Future |
| 执行器 | 调度任务并推进其顶层 Future；本项目通常使用 Tokio | 每条 SSE 事件对应一个线程 |

`async fn` 调用返回 Future；其执行状态可包含跨 `.await` 需要保存的局部变量、参数和持有的资源。业务数据由 Future 或相关对象持有，具体取决于捕获、借用和所有权；不会因为传入 `cx` 就自动放进 `cx`。

## 从 Pending 到再次推进

下面是定时器等待的概念流程，实际组件可以把唤醒注册委托给底层资源：

```text
执行器 poll 任务 → Future poll Sleep(cx)
                           │
                     尚未到期：登记唤醒
                           │
                     返回 Pending，任务让出执行机会
                           │
定时器到期 → 使用 Waker 通知执行器
                           │
执行器稍后再次 poll 任务 → Sleep 可能返回 Ready
```

唤醒是一次调度通知：不保证立即执行，也不保证下次一定 `Ready`。网络和定时器都可能唤醒同一个任务；具体能否推进，取决于再次检查时的条件。`Pending` 时，组件需要确保将来可能推进时会收到合适的唤醒；如果委托给子 Future 或 I/O，子组件可以负责登记。

每次 `poll` 传入的 `cx` 可能不同；手写组件不能假定以前保存的 Waker 永远有效。不要在 `poll` 中阻塞线程等待事件。Future 返回 `Ready` 后，调用方不应继续 poll 同一个已完成的 Future。普通 `.await` 会处理推进细节；不是每次 `.await` 都会挂起。

## 如何保存 Waker

手写 Future 并自行管理等待事件时，常用 `cx.waker().clone()` 保存当前任务的通知方式，供事件发生时使用：

```rust
let waker = cx.waker().clone(); // 取得可以独立保存的 Waker
// 把 waker 保存到事件的共享状态中；事件发生后：
waker.wake(); // 通知执行器，任务可以再次被 poll
```

`cx.waker()` 借用 Waker；`clone()` 得到可保存的 Waker。`wake()` 消耗它，`wake_by_ref()` 则借用它发出通知。上面只是说明操作，并非完整的登记实现：实际代码要同步检查事件和登记 Waker，避免两者之间错过事件，并在后续 poll 时使用最新的 Waker。

调用 `poll(cx)` 本身不会让执行器自动登记唤醒。返回 `Pending` 的组件负责安排唤醒，也可以委托给被 poll 的子组件。项目中使用的 `Sleep` 已经处理了登记，项目代码不需要自己保存它的 Waker。

## 项目中的写入超时

```text
WriteTimeoutIo
├─ inner: TcpStream      ← 网络连接
├─ write_timer: Sleep    ← 写入停滞后的截止时间（Option 中）
└─ poll_write(cx)        ← 检查计时器，再尝试写 socket
```

1. socket 写入返回 `Pending`，首次停滞时创建 `Sleep`，保存在 `self.write_timer`。
2. `check_timeout` 用当前 `cx` poll 保存的 `Sleep`；未到期时，Sleep 安排到期后唤醒当前任务。
3. 网络可写或计时器到期都可能触发唤醒。任务再次运行后，代码先检查计时器，再尝试写入。
4. 计时器到期返回超时错误；真正写出正数字节时清除计时器，下一次停滞重新计时。连续 `Pending` 不重置截止时间。

`check_timeout` 的 `Ok(())` 只表示没有检测到超时，不代表写入已经成功。计时器和 socket 是两个独立状态，都能使用当前任务的 Waker。

## 自查

1. `Pending` 后为什么不能假设运行时会按固定周期扫描它？
2. `cx`、`Sleep` 和业务 Future 分别保存什么？
3. 如果 socket 可写触发唤醒，为什么下一次仍可能是 `Pending`？

参考：[Future::poll](https://doc.rust-lang.org/std/future/trait.Future.html) · [Context](https://doc.rust-lang.org/std/task/struct.Context.html) · [Waker](https://doc.rust-lang.org/std/task/struct.Waker.html)。
