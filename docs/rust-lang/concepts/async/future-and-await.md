# Future 与 await：创建、暂停和继续

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**调用 async 函数先得到 Future，轮询才执行函数体；await 暂停当前任务，不等于阻塞线程。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](../types/generics-and-inference.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) · [元组、元组结构体与 unit](../values/tuples-and-unit.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`operation().await` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 创建与执行的时间点

```text
调用 async 函数 → 生成 Future，保存参数和所需状态
轮询 Future     → 执行到需要等待的位置
等待就绪        → 被唤醒，再继续轮询
Ready           → 得到 Output
```

普通 async 函数不会因为“调用”就自动在后台启动。Tokio spawn 会把 Future 交给运行时独立调度；这与直接 await 是不同的组织方式。

| 操作 | 做什么 |
|---|---|
| 创建 Future | 得到描述后续工作的值 |
| `.await` | 在当前异步任务中驱动它，必要时暂停当前任务 |
| `tokio::spawn(future)` | 创建由运行时独立调度的任务 |

“暂停当前任务”不等于把整个 OS 线程睡住；但如果 Future 内部调用阻塞函数，线程仍然可能被阻塞。

## pending：泛型类型与函数调用

**这里指定的是 Future 输出类型的泛型参数，不是函数入参类型。** 先看标准库的函数签名：

```rust
pub fn pending<T>() -> Pending<T>
```

| 位置 | 含义 |
|---|---|
| `<T>` | 声明一个泛型类型参数 |
| 函数名后面的 `()` | 没有运行时入参 |
| `-> Pending<T>` | 函数立即返回的类型是 `Pending<T>` |
| `Pending<T>` 中的 T | 这个 Future 的输出类型 |

先选择熟悉的整数类型，再换成 unit 类型：

```rust
let number_future = std::future::pending::<u32>();
let unit_future = std::future::pending::<()>();
```

| 调用 | 类型参数 | 函数立即返回 | Future 的输出类型 |
|---|---|---|---|
| `pending::<u32>()` | `T = u32` | `Pending<u32>` | `u32` |
| `pending::<bool>()` | `T = bool` | `Pending<bool>` | `bool` |
| `pending::<()>()` | `T = ()` | `Pending<()>` | `()` |

`()` 是 unit 类型，表示不携带额外信息，它唯一的值也写作 `()`。
在 `pending::<()>()` 中，尖括号里面的 `()` 是类型，最后的 `()` 才是函数调用。

```text
std::future::pending :: <()> ()
└────函数路径──────┘    │    └─ 调用函数，参数列表为空
                      └─ 泛型类型参数是 unit 类型 ()
```

| 部分 | 含义 |
|---|---|
| `std::future::pending` | 标准库 future 模块的泛型函数 |
| `::<()>` | 指定类型参数 T 为 unit 类型 `()` |
| 最后的 `()` | 调用函数，不传入参数 |
| 整个表达式 | 返回 `Pending<()>`，不是直接返回 unit 值 |

这个 Future 永远不会就绪，所以 `.await` 会一直等待；`()` 只是预期输出的类型，
不是说等待后真的会得到一个值。标准库说明见
[pending](https://doc.rust-lang.org/std/future/fn.pending.html)。
它在本次测试中用来保持“任务持有 guard、等待中”的状态；具体场景见
[Tokio 异步任务与取消测试](../../libraries/tokio/cancellation.md)。

```text
pending::<()>()  → 创建 Future，立即返回这个 Future
       .await   → 等待它完成，但它永远不会完成
```

选 `()` 表示不需要输出数据。一直等待是 `pending` 的行为，不是 `()` 的作用；
换成 `pending::<u32>().await` 也一样不会完成。

## Future、async 与 await

| 写法 | 含义 | 容易误解的地方 |
|---|---|---|
| 调用 `async fn` | 得到一个 Future | 调用本身不等于已经执行完函数体 |
| `async { ... }` | 把这段工作描述为 Future | 不会自动创建后台任务 |
| `future.await` | 等待它产生结果 | 不会自动创建线程；若结果已就绪，也不必挂起 |

Future 被轮询时才能推进。尚未就绪时，它返回等待状态；等资源就绪，任务被唤醒后再继续。
普通异步 I/O 等待不会一直占着线程。不是所有 `.await` 都必然让出执行机会。
参见 [标准库 Future](https://doc.rust-lang.org/std/future/trait.Future.html)。

`.await` 也支持实现 `IntoFuture` 的值：不要把“能 await”机械理解为“这个类型直接实现了 Future”。
例如 Axum 0.8 的 `axum::serve(...)` 返回 `Serve`；等待它是在持续运行服务器，不是等一条请求。
普通 `serve(...).await` 不会在处理完一次请求后退出。参见 [Axum serve](https://docs.rs/axum/0.8.9/axum/fn.serve.html)。

补充类型层面的读法：Future 的完成结果由关联类型 Output 表示。
`Future<Output = Result<HttpResponse, HttpResponse>>` 完成后得到整个 Result；
timeout 包装后再产生外层 Result，因此可能出现 Ok(Err(...))。
见 [HttpResponse、Output 与重试签名](../types/aliases-and-newtypes.md)。

## pending：故意让任务一直等

`std::future::pending::<()>()` 来自标准库，返回一个永远不会就绪的 Future。
`::<()>` 是 turbofish，指定预期输出类型为 unit；并不意味着它真的会返回这个值。
在取消测试里，它让任务停在“持有 guard、等待中”的状态，不会自己正常完成。
参见 [标准库 pending](https://doc.rust-lang.org/std/future/fn.pending.html)。

## 自查与关联

创建 Future 与执行它是同一时间吗？

[async move：把哪些值交给 Future](async-capture.md)
