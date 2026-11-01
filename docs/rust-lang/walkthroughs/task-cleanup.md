# 代码导读：ManagedTask 的清理责任

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**先分清拥有权转移、局部可变性和异步执行时机，再看 take、abort 与 await。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [self、mut self、&self 与 &mut self](../concepts/ownership/method-receivers.md) · [Option 中的借用与取值：as_ref、as_deref、take](../concepts/errors/option-access-and-take.md) · [取消：abort 之后为什么还要 await](../libraries/tokio/cancellation.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../src/gateway/tests/support.rs)：`ManagedTask / TestServer` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先分清谁持有谁

```text
TestServer
├── address：String
└── task：ManagedTask<()>
           └── handle：Option<JoinHandle<()>>
```

ManagedTask 自己实现 Drop；TestServer 没有自定义 Drop，但字段仍会自动销毁，从而触发内层 ManagedTask 的兜底取消。
下文只讲项目组合：普通接收者规则见方法接收者文章，Box 对照例子见 Option 文章。

## 消费自身与 Option::take：ManagedTask 的例子

**先拆开三个问题：对象归谁、谁能修改、代码什么时候执行。**

## `mut self`：对象交给方法，方法内部可以修改它

| 参数 | 对象归谁 | 方法能否修改对象 |
|---|---|---|
| `self` | 所有权转给方法 | 不能通过此绑定直接修改普通字段 |
| `mut self` | 所有权转给方法 | 能 |
| `&mut self` | 仍归调用方，方法只是可变借用 | 能 |

```text
mut self
 │    └── 整个对象交进来
 └─────── 方法内部允许修改这个对象
```

`my-smg` 测试中的 `cancel_and_join(mut self)` 接收整个 `ManagedTask`。
这里需要 `mut`，是因为 `self.handle.take()` 会把字段从 `Some(句柄)` 改成 `None`。
**`mut self` 是拥有对象，`&mut self` 是借用对象，两者不能混为一谈。**

## 为什么调用方不需要 `let mut task`

项目测试用普通的 `let task` 保存管理对象，然后调用 `task.cancel_and_join().await`。

```text
调用方的 task                 方法内部的 mut self
      │                              ↑
      └──────── 对象所有权转移 ────────┘
```

**调用方没有修改自己的 `task`，而是把对象交出去了。** 接收方可以用一个允许修改的新绑定接住它，
原绑定是否为 `mut` 不影响这件事。就像 String 可以从一个不可变绑定移动到另一个可变绑定，
原拥有者不能修改，不代表新拥有者也不能修改。

`ManagedTask` 不是 `Copy` 类型，所以交出后，原来的 `task` 不能再使用。

## 异步方法：交出对象和执行方法体是两个时间点

把原调用拆成两步看，下面只是调用时序示意：

```rust
let future = task.cancel_and_join(); // 对象已经交给 future
// 此时不能再使用 task

let result = future.await;          // 驱动方法体执行，等待结果
```

```text
调用前
task ──拥有──> ManagedTask

调用 cancel_and_join()
future ──拥有──> ManagedTask
task 已失效，方法体尚未执行

执行 future.await
取出句柄 → 请求取消 → 等待任务结束 → 返回结果
```

异步函数调用时把参数捕获到返回的 Future 中；方法体在 Future 被轮询时执行，
不是调用时就执行，也不是等到 `.await` 完成才转移所有权。
见 [Rust：异步函数](https://doc.rust-lang.org/reference/items/functions.html#async-functions)。

**调用 `cancel_and_join()` 时，就把管理对象交出去了；`mut` 允许方法内部修改它，
`.await` 则负责驱动取消和等待流程。** 这里新创建的是“取消并等待”的 Future，
被管理的后台任务此前已由 `tokio::spawn` 启动，两者要分开理解。

## `Option::take`：取出句柄，留下合法状态

管理对象的字段是 `Option<JoinHandle<T>>`，处理流程是：

```text
self.handle = Some(句柄)
       │ take()
       ├─ 返回 Some(句柄)，句柄转给局部变量
       └─ self.handle 变成 None
                  ↓
          self 最后 Drop 时不再处理该句柄
```

`take()` 的签名是 `fn take(&mut self) -> Option<T>`。它移动值，不克隆；原来是 `None` 时，
返回值也是 `None`。见 [标准库 Option::take](https://doc.rust-lang.org/std/option/enum.Option.html#method.take)。

## 回到 ManagedTask：句柄能借用、能取走，但不能解引用

换成 JoinHandle 时，`take()` 同样转移的是已有句柄，不是新启动一个任务。

对照普通借用方法：`worker.id()` 借用 Worker，调用后原对象仍能使用。
`cancel_and_join(mut self)` 属于后者，`mut` 不改变消费语义。

这里不能直接写 `let handle = self.handle`：`ManagedTask` 实现了 `Drop`，Rust 不允许直接
移出这种类型的非 Copy 字段，避免析构逻辑面对一个缺字段的对象。`take()` 通过可变借用
换出完整字段值、留下合法的 `None`，因此可行；不是绕过所有权检查。

| 写法 | 得到什么 | 原字段 |
|---|---|---|
| `&self.handle` | `&Option<JoinHandle<T>>`，借用整个字段 | 不变 |
| `self.handle.as_ref()` | `Option<&JoinHandle<T>>` | 不变 |
| `self.handle.as_deref()` | 不能调用：`JoinHandle<T>` 没有实现 `Deref` | — |
| `self.handle.take()` | 拥有型 `Option<JoinHandle<T>>` | `None` |

**`as_deref()` 不是剥掉任意类型的外壳。** `Box<T>` 可以解引用到 T，
但 `JoinHandle<T>` 的 T 是任务返回值，要等待句柄完成才能得到 `Result<T, JoinError>`，
不能通过解引用取得。见 [Tokio：JoinHandle](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinHandle.html)。

字段部分移动的一般规则见 [Move 与部分移动](../concepts/ownership/move-and-partial-move.md)；异步取消、字段自动析构与
显式等待的配合见 [Tokio：任务资源管理](task-cleanup.md)。

## 与已有文档的联系

| 本专题问题 | 继续阅读 |
|---|---|
| Arc 克隆为何还是同一个 Worker | [Arc、Weak 与循环引用](../concepts/sharing/weak-and-cycles.md) |
| guard 借用为何不能比 Worker 活得更久 | [guard 的生命周期](../concepts/ownership/lifetimes.md) |
| 计数如何通过共享引用修改 | [AtomicUsize 与内部可变性](../concepts/sharing/atomics.md) |
| 策略的 MutexGuard 为何要在 await 前释放 | [锁与 await](../concepts/sharing/mutex-and-guards.md) |
| `pending::<()>()` 的尖括号和括号是什么 | [pending 的语法拆解](../concepts/async/future-and-await.md) |
| `Result`、`expect_err` 如何理解 | [Result 与错误传播](../concepts/errors/question-mark.md) |
| `mut self` 与 take 为什么一起使用 | [消费自身与字段转移](task-cleanup.md) |
| `impl FnMut() -> bool` 和双层 move 是什么 | [闭包与 Fn 系列](../concepts/types/closures-and-fn.md) |
| post 的处理器为什么返回 Future，MethodRouter 又是什么 | [Axum 语法与 Handler 约束](../libraries/axum/router-and-handler.md) |

普通 `InFlightGuard` 不持有锁；它跨越 await 是为了覆盖请求等待期间。
标准库 `MutexGuard` 则持有锁，作用和约束不同，不能只因名字都含 guard 就混为一谈。

## 测试重构：抽准备工作，不隐藏行为

复用不是把每一段相似代码都藏起来，而是把稳定机制和每个场景的决策分开。

| 原则 | my-smg 中的例子 | 仍放在测试中的内容 |
|---|---|---|
| 准备过程独立于被测行为 | 直接向注册表插入已知 Worker | 调用 add_worker，验证重复注册不替换对象 |
| 机制与条件分离 | wait_until 管理循环和期限 | 闭包明确判断计数为 1、2 或健康恢复 |
| 所有权跟随清理责任 | TestServer 拥有 ManagedTask | 正常路径明确 stop，再检查结果 |
| 释放机制不等于完成保证 | Drop 请求 abort | await 或有界观察确认取消已经生效 |
| 抽象不能丢失失败信息 | stop 允许取消，但保留其他 JoinError | 取消测试单独断言 is_cancelled |

例如重复注册测试：

```text
准备：直接注册 original Worker
执行：调用 add_worker，使用相同 ID、不同地址
验证：返回 409；注册表仍指向 original；旧地址没有改变
```

如果准备阶段也调用 add_worker，准备和执行都依赖同一被测函数，就更难判断失败发生在哪里。
不是说任何测试都不能通过接口准备，而是本例有更独立、简单的注册表准备方式。

两个任务共享一个 Worker 的场景也必须留清楚：

```text
启动 task_a 和 task_b → 同一 Worker 的计数 2
取消并等待 task_a    → 计数 1
取消并等待 task_b    → 计数 0
```

任务变量命名为 task_a_worker、task_b_worker，表示两个任务持有的 Arc，而不是两个不同节点。
不要把这三步收进一个笼统的“测试通过”辅助函数，否则阅读者看不见正在验证的行为。

具体语法见 [闭包专题](../concepts/types/closures-and-fn.md)、
[容器借用与 take](../concepts/errors/option-access-and-take.md)、
[Tokio 的资源管理实例](task-cleanup.md)。

## 与条件等待及测试设计的关系

`wait_until` 负责循环、间隔和总等待期限；闭包负责判断“什么叫就绪”。
例如判断计数为 1、计数为 2，或健康状态已恢复，可以复用相同的等待机制。

闭包返回 false 时继续等待，返回 true 时结束；等待超时返回 Elapsed。
这个闭包是同步检查，不要在里面执行阻塞网络请求或试图直接使用 await。
如果检查本身需要异步操作，那是另一个接口需求，本例没有引入。

关联阅读：

- [Tokio：任务资源管理与等待条件](task-cleanup.md)。
- [设计原则：机制与断言保持分离](task-cleanup.md)。

## 自查与关联

Drop 的兜底和显式 cancel_and_join 分别保证什么？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：配置加载与分层错误](configuration-loading.md)
