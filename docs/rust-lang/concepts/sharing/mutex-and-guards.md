# Mutex 与 Guard：锁何时释放

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**lock 成功返回守卫，守卫销毁时解锁；锁内做短操作，不持标准库 MutexGuard 跨 await。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md) · [Drop 与 RAII：离开作用域如何收尾](../ownership/drop-and-raii.md) · [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`state.registry.lock()` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 为什么需要 `Arc<Mutex<T>>`

多个 Axum 请求会拿到克隆后的 `AppState`。如果状态中保存：

```rust
failures_remaining: Arc<Mutex<usize>>
```

克隆 `AppState` 时，只会克隆 `Arc`；所有副本仍然指向同一个 `Mutex<usize>`：

```text
请求 A ──Arc──┐
请求 B ──Arc──┼──> Mutex<usize>
请求 C ──Arc──┘
```

两层类型职责不同：

| 类型 | 解决的问题 |
|---|---|
| `Arc<T>` | 多个任务共同拥有同一个 `T` |
| `Mutex<T>` | 对 `T` 的可变访问必须串行进行 |

`Arc<T>` 本身不允许多个任务安全地同时修改 `T`。`Mutex<T>` 也不负责多个长期所有者，因此并发
服务中经常组合成 `Arc<Mutex<T>>`。

`Arc` 的所有权与引用计数见 [`Arc`、`Weak` 与循环引用](weak-and-cycles.md)。
当 Mutex 保护的是需要保存轮询进度的 `Box<dyn Policy + Send>` 时，见
[配置驱动的共享策略](../../walkthroughs/policy-selection.md)。
如果只是并发更新单个整数，也可对照
[`AtomicUsize` 与内部可变性](atomics.md)，理解无需加锁的计数方式。

## `MutexGuard<'_, usize>` 的两个参数

```text
MutexGuard<'生命周期, 数据类型>
```

在：

```rust
MutexGuard<'_, usize>
```

| 参数 | 种类 | 含义 |
|---|---|---|
| `'_` | 生命周期参数 | 由编译器推断 Guard 对 Mutex 的借用时间 |
| `usize` | 类型参数 | Mutex 内部保护的数据类型 |

`'_` 不是类型，也不是 `'static`。它表达的约束是：

```text
Mutex 存活并被借用
        │
        └─ MutexGuard 不能比这个 Mutex 活得更久
```

生命周期的通用概念见 [`Formatter<'_>`、生命周期、`Display` 与 `Error`](../errors/debug-display-and-formatter.md)。

## 为什么用额外的大括号

失败计数代码写成：

```rust
let should_fail = {
    let mut remaining = match state.failures_remaining.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if *remaining == 0 {
        false
    } else {
        *remaining -= 1;
        true
    }
};

tokio::time::sleep(state.delay).await;
```

大括号同时完成两件事：

```text
内部 block 的最后一个表达式
        │
        └─ 生成 should_fail: bool

离开内部 block
        │
        └─ remaining 被 drop，Mutex 自动解锁
```

这是 RAII：资源的释放绑定到值的生命周期，而不是手动调用 `unlock()`。

## 为什么不能持有 Guard 跨越 `.await`

错误的时间线：

```text
请求 A：lock，取得 Guard
        │
        ├─ sleep(...).await：A 暂停，但 Guard 仍存在
        │
请求 B：调用 std::sync::Mutex::lock()
        │
        └─ 锁仍被 A 占用，B 阻塞 Tokio 工作线程
```

`sleep().await` 本身不阻塞线程。真正的问题是：A 暂停时仍占着锁，其他任务调用标准库的阻塞
`lock()` 后会占住 Tokio 工作线程。

如果多个工作线程都阻塞在同一把锁上，负责唤醒 A 的异步任务可能无法及时继续，导致吞吐下降、
任务饥饿，严重时形成类似死锁的停滞。

另外，`std::sync::MutexGuard` 不能安全地在线程之间移动。Guard 跨越 `.await` 时，handler 生成的
Future 往往不满足 Axum 所需的 `Send`，代码可能直接无法编译。

正确边界是：

```text
lock
  v
只做快速的同步读写
  v
drop Guard，释放锁
  v
网络、sleep 等 .await
```

`tokio::sync::Mutex` 允许 Guard 跨越 `.await`，但“允许”不代表应该长时间持锁。只保护普通内存
计数器且临界区很短时，可以使用 `std::sync::Mutex`，前提是必须在 `.await` 前释放 Guard。

## 自查与关联

额外的大括号如何限制持锁时间？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
