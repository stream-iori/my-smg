# 扩展：上游的 Arc 与 Weak 关系

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**这是独立实现完成之后的对照材料；上游版本和本地简化设计要区分。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [learning/04-source-map.md](../../../learning/04-source-map.md)：`上游对照入口` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 原项目为什么选择 Arc

| 源码事实 | 设计效果 |
|---|---|
| [WorkerRegistry::get](../../../../sgl-model-gateway/src/core/worker_registry.rs#L373) 返回克隆的 Arc | 查询结束后不必继续持有注册表条目的借用或锁 |
| [WorkerRegistry::remove](../../../../sgl-model-gateway/src/core/worker_registry.rs#L316) 移除条目与索引，并标记不健康 | 更新后续查询和选择所看到的节点资格，而不是立即销毁所有请求中的对象 |
| [WorkerLoadGuard](../../../../sgl-model-gateway/src/core/worker.rs#L1147) 拥有 `Arc<dyn Worker>` | 即使注册表移除节点，guard 仍能在结束时减计数 |
| guard 可随响应体存活 | 流式响应持续时，不依赖处理函数的局部借用 |

这不是“Arc 比引用更高级”，而是需要请求的所有权独立于注册表借用。
`dyn Worker` 负责具体实现的抽象，`Arc` 负责共同拥有，两个问题不要混为一谈。
`Arc` 也不会自动同步可变字段、停止远端进程或撤销已经发出的请求。

设计感悟见 [realization：可选资格与对象存活分离](../walkthroughs/worker-lifecycle.md)。
引用与生命周期的编译期约束见
[guard 借用 Worker](../concepts/ownership/lifetimes.md)。

## SGL Model Gateway 中的例子

SMG 启动时创建 `AppContext` 后，创建 JobQueue：

```rust
let app_context = Arc::new(...);
let weak_context = Arc::downgrade(&app_context);
let worker_job_queue = JobQueue::new(JobQueueConfig::default(), weak_context);
```

源码见 [`server.rs:810-823`](../../../../sgl-model-gateway/src/server.rs#L810-L823)。关系是：

```text
AppContext ──Arc──> JobQueue
     ▲                  │
     └──────Weak────────┘
```

如果 JobQueue 反过来持有 `Arc<AppContext>`，就会形成强引用环：

```text
AppContext ──Arc──> JobQueue
     ▲                  │
     └──────Arc─────────┘  <- 无法释放
```

用 `Weak<AppContext>` 后，关闭服务并释放最后一个 `Arc<AppContext>` 时，JobQueue 不会阻止
AppContext 析构。

## 最小可运行模型

```rust
use std::sync::Arc;

let strong = Arc::new(String::from("hello"));
let weak = Arc::downgrade(&strong);

assert_eq!(Arc::strong_count(&strong), 1);
assert_eq!(Arc::weak_count(&strong), 1);

assert_eq!(weak.upgrade().as_deref(), Some("hello"));

drop(strong);

assert!(weak.upgrade().is_none());
```

`drop(strong)` 后，字符串已被释放；变量 `weak` 仍然存在，但它只能返回 `None`。

## 实际类型链

[`RouterManager`](../../../../sgl-model-gateway/src/routers/router_manager.rs) 中的字段类型是：

```rust
default_router: Arc<std::sync::RwLock<Option<RouterId>>>
```

读取后，各层类型如下：

```text
Arc<RwLock<Option<RouterId>>>
             │ .read()
             ▼
Result<RwLockReadGuard<Option<RouterId>>, PoisonError<_>>
             │ .unwrap_or_else(...)
             ▼
RwLockReadGuard<Option<RouterId>>       ← default_router
             │ *
             ▼
Option<RouterId>
             │ Some(ref default_id)
             ▼
&RouterId                               ← default_id
```

## 为什么使用 `*`

局部变量 `default_router` 不是 `Option<RouterId>`，而是：

```rust
RwLockReadGuard<'_, Option<RouterId>>
```

`RwLockReadGuard<T>` 实现了 `Deref<Target = T>`。因此：

```rust
*default_router
```

表示访问守卫后面的 `Option<RouterId>`。这里不会释放锁；只要守卫还在作用域内，读锁
就仍然有效。

锁守卫如何通过 `Deref/DerefMut` 访问内部数据、如何用 `Drop` 自动解锁，以及毒锁的含义，见
[`Mutex`、`MutexGuard`、解引用与毒锁](../concepts/sharing/mutex-and-guards.md)。

模式匹配不会像方法调用那样自动穿透 `RwLockReadGuard`，所以原写法需要显式 `*`。

## 为什么使用 `ref`

如果写成：

```rust
if let Some(default_id) = *default_router {
```

`default_id` 会按值绑定，Rust 会尝试把 `RouterId` 从锁保护的数据中移出来：

```text
Option<RouterId>（锁中的共享数据）
       │ move RouterId
       ▼
default_id: RouterId
```

读锁只允许共享读取，不允许取走内部数据，因此编译器会拒绝这种移动。

加上 `ref` 后，模式绑定改为借用：

```text
Option<RouterId>（锁中的共享数据）
       │ borrow
       ▼
default_id: &RouterId
```

注意：`ref` 只改变模式变量的绑定方式，它不是变量类型的一部分。

```rust
Some(ref default_id) // default_id: &RouterId
Some(default_id)     // default_id: RouterId，默认按值绑定
```

## 写法一：使用 `as_ref()`

```rust
let default_router = self
    .default_router
    .read()
    .unwrap_or_else(|e| e.into_inner());

if let Some(default_id) = default_router.as_ref() {
    self.routers.get(default_id).map(|r| r.clone())
} else {
    None
}
```

类型变化很直接：

```text
Option<RouterId> --as_ref()--> Option<&RouterId>
```

方法调用会自动解引用 `RwLockReadGuard`，找到 `Option::as_ref()`。

## 三种写法对照

| 写法 | `default_id` 类型 | 是否移动数据 | 建议 |
|---|---|---:|---|
| `Some(default_id) = *guard` | `RouterId` | 是，无法从读锁守卫中移动 | 不使用 |
| `Some(ref default_id) = *guard` | `&RouterId` | 否 | 正确，但不够直观 |
| `Some(default_id) = guard.as_ref()` | `&RouterId` | 否 | 推荐 |

## 自查与关联

当前 guard 的借用设计与上游共享拥有设计有什么不同？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：thiserror 的错误派生](thiserror.md)
