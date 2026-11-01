# async move：把哪些值交给 Future

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**move 改变捕获方式，引用也能被按值捕获；保存执行状态使局部数据可以跨等待存活。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Future 与 await：创建、暂停和继续](future-and-await.md) · [闭包与 Fn、FnMut、FnOnce](../types/closures-and-fn.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`operation 闭包` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 不写 move，也可能移动变量

| 写法 | 捕获规则 |
|---|---|
| `async { ... }` | 根据块内用法，逐个推断共享借用、可变借用或按值捕获 |
| `async move { ... }` | 按值捕获块内使用到的外部变量 |

例如下面只读取 String，可以借用；当前函数直接等待，能保证借用期间 message 有效：

```rust
async fn print_message() {
    let message = String::from("hello");

    async {
        println!("{message}");
    }
    .await;
}
```

如果块内消费 String，编译器会推断移动，即使没有写 move：

```rust
let message = String::from("hello");
let future = async {
    drop(message);
};
// message 已移入 future，之后不能再使用。
```

`my-smg` 测试服务中的 `axum::serve(listener, app)` 同样按值取得 listener 和 app，
因此这段 async 块可以省略 move；保留 `async move` 则明确表达后台任务持有这些资源。
这不是“spawn 总能省略 move”，而是本次块内用法已经足以推断移动。

捕获规则见 [Rust Reference：async blocks 的捕获方式](https://doc.rust-lang.org/reference/expressions/block-expr.html#capture-modes)。

## 什么时候需要 move

只读取变量时，默认捕获通常是借用。交给 spawn 的任务可能比当前函数活得久，
不能依赖当前函数中局部 String 的借用，因此要把所有权交给 Future：

```rust
let message = String::from("hello");

let task = tokio::spawn(async move {
    println!("{message}");
});
```

| 场景 | 如何判断 |
|---|---|
| 当前函数直接 `.await`，外部变量一直有效 | 通常可以借用，不必强制 move |
| 后台任务使用外部拥有型数据 | 通常用 move，让任务独立拥有数据 |
| 块内本来就按值消费变量 | 不写 move 也可能推断为移动 |
| 调用方和后台任务都需要同一个 Worker | 先 Arc::clone，再将那份 Arc move 进去 |

move 对不同值的效果也不同：

| 捕获值 | 效果 |
|---|---|
| `String` 等非 Copy 值 | 移动所有权，外面的原变量不能继续使用 |
| `usize` 等 Copy 值 | 捕获副本，原变量仍能使用 |
| `Arc<Worker>` | 移动这份 Arc，不自动增加强引用计数 |
| `&Worker` | 捕获引用，不取得 Worker 的所有权，也不延长它的存活时间 |

move / Copy / 借用的基础见 [所有权、转移与借用](../ownership/move-and-partial-move.md)。

## Future 持有变量，为什么暂停后仍存在

可以把 Future 理解成保存执行进度的对象。编译器会让它保存恢复执行所需的捕获值、
跨越等待点的局部变量，以及正在等待的异步操作；这不是每暂停一次就重新创建变量。
具体布局由编译器决定，不需要手写状态机。

```text
创建 async move 块
    ↓
Future 持有捕获值：例如 Arc<Worker>
    ↓ 第一次推进
创建 guard：在途计数 +1
    ↓ 等待未就绪
Future 保存继续执行所需的 guard 等状态；线程可执行其他任务
    ↓
继续执行，或取消并销毁 Future
    ↓
guard Drop：在途计数 -1
```

所以普通在途 guard 可以跨越 await 保留，覆盖请求等待期间。它与持有锁的 MutexGuard
职责不同，见 [锁守卫与 await](../sharing/mutex-and-guards.md)。
Future 的保存状态机制见 [Rust Reference：async Future 的布局说明](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks)。

## 外层 move 闭包与内层 async move

延迟聊天服务器的路由使用：

```rust
post(move || async move {
    tokio::time::sleep(delay).await;
    "slow response"
})
```

这里有两个不同的值：负责接收调用的闭包，以及每次调用时创建的 Future。

```text
外部 delay
    ↓ 外层 move：按值捕获
路由处理闭包持有 delay
    ↓ 每次调用闭包，创建内层 async move
本次请求的 Future 持有 delay
    ↓ 被轮询后
等待时间，再返回响应
```

本例的 Duration 实现 Copy，内层按值捕获取得副本，不会把闭包中的 delay 永久取走。
所以同一个路由处理闭包可以重复调用。

不要推广成“加两个 move 就能重复使用任意值”：如果内层 Future 取走了外层闭包持有的
非 Copy String，而调用时没有另行 clone，外层闭包通常只能按 FnOnce 的方式调用。
`move` 决定捕获方式；Fn 系列能力还取决于调用时如何使用捕获值。
见 [Rust Reference：闭包能力与捕获](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions)，
以及 [Tokio：async move 捕获规则](async-capture.md)。

## 闭包与异步块不是同一种值

| 写法 | 创建什么 | 如何使用 |
|---|---|---|
| `move \|\| { ... }` | 闭包 | 调用它，例如 handler() |
| `async move { ... }` | Future | 驱动它，例如 .await |

`post(move || async move { ... })` 中，post 的参数是闭包；闭包调用后返回 Future。
post 自己返回 MethodRouter，不是里面那个闭包。
判断依据是闭包语法，以及 Handler 的 Future 约束。
完整拆解见 [Axum：post、Handler 与 Future](../../libraries/axum/router-and-handler.md)。

## 自查与关联

move 进去的是对象还是引用，要看什么？

[Future 与 await：创建、暂停和继续](future-and-await.md)
