# 泛型、类型推断与 turbofish

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**泛型把类型留作参数；::<类型> 显式填写类型，圆括号才传运行时数据。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [数组、切片、Vec 与 usize](../values/arrays-slices-and-vec.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`ManagedTask<T>` |
| 阅读时机 | 阶段 3 读到基础自查；Future 和任务应用按后文阶段阅读。 |

## 对照：输入和输出都使用泛型参数

**`::<类型>` 本身只表示指定泛型参数；这个参数用于入参、返回值还是两者，要看函数定义。**
下面是独立的语法示例，不是项目实现：

```rust
fn identity<T>(value: T) -> T {
    value
}
```

```text
identity<T>(value: T) -> T
         │        │     │
         │        │     └── 返回值类型也是 T
         │        └──────── 入参类型是 T
         └───────────────── 声明泛型参数 T
```

它不转换数据，只把接收到的值原样返回。选择 `u32` 时：

```rust
let number = identity::<u32>(42);
```

| 部分 | 含义 |
|---|---|
| `::<u32>` | 编译期指定 `T = u32` |
| `(42)` | 调用函数，传入运行时的值 42 |
| `number` | 得到返回的 `u32` 值 42 |

换成拥有型数据时，还能看到所有权转移：

```rust
let original = String::from("hello");
let result = identity::<String>(original);
// original 已移动；result 拥有原来的字符串，没有复制内容
```

```text
original ──move──> value（函数参数）──move──> result
                     T = String
```

本例也可根据实参推断类型，写成 `identity(original)`；显式写 `::<String>` 是为了展示泛型参数。

泛型参数决定类型，运行时参数携带值。输入输出都是 T，意味着二者使用同一个具体类型。
上下文能推断类型时不必总写 turbofish，详见[类型推断](type-inference.md)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | identity 接收 String 后，original 还能直接使用吗？ |
| 解释 | 为什么 T 是类型参数，而 value 是运行时参数？ |
| 边界 | 函数承诺返回同一个 T，能随意把 String 换成整数返回吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 项目应用：阶段 8～9 的泛型任务

先读[Option](../errors/option-and-result.md)、[Drop](../ownership/drop-and-raii.md)、[JoinHandle](../../libraries/tokio/join-handle.md)。下文省略可见性或方法体，不能作为完整任务管理器编译。

## 泛型结构体与泛型 impl

泛型不只用于函数，也可以表示“结构相同、内部数据类型不同”的结构体。
`my-smg` 测试中的定义是：

```rust
struct ManagedTask<T> {
    handle: Option<JoinHandle<T>>,
}
```

这里 `T` 是任务成功时的返回值类型，不是线程类型，也不是必然指向 Worker。

| 具体类型 | 管理的任务 |
|---|---|
| `ManagedTask<()>` | 不返回额外数据的服务器或计数测试任务 |
| `ManagedTask<(StatusCode, String)>` | 返回状态码和正文的 chat 任务 |

```rust
impl<T> ManagedTask<T> { /* 为所有 T 提供固有方法 */ }
impl<T> Drop for ManagedTask<T> { /* 为所有 T 实现 Drop trait */ }
```

| 部分 | 作用 |
|---|---|
| `impl<T>` | 声明本次实现使用的类型参数 T |
| `ManagedTask<T>` | 指定为哪一组具体类型提供实现 |
| `Drop for` | 实现 trait，而不是定义普通固有方法 |

类型参数名字不必叫 T；没有写 bound，也不表示它自动具有 `Clone`、`Send` 等全部能力。
调用 `ManagedTask::new(handle)` 时，编译器可以从 `JoinHandle<T>` 推断 T，不必手写 turbofish。
泛型本身不要求堆分配或动态分发。

关联阅读：[接收者与 take](../../walkthroughs/task-cleanup.md)、
[闭包参数中的 impl Trait](closures-and-fn.md)、
[Tokio 资源管理实例](../../walkthroughs/task-cleanup.md)。

## 延伸：阶段 5 的 Future 输出参数

先读[Future](../async/future-and-await.md)。

| 函数签名 | T 的使用位置 | 指定类型的含义 |
|---|---|---|
| `fn consume<T>(value: T)` | 入参 | 实参类型要符合 T |
| `fn pending<T>() -> Pending<T>` | 返回的 Future | Future 的输出类型是 T |
| `fn identity<T>(value: T) -> T` | 入参与返回值 | 输入和输出使用同一个类型 T |

因此，说 pending 的 T 是“出参的泛型”时，更准确地说：**T 是返回的 Future 的输出类型，
函数本身直接返回的是 `Pending<T>`，而不是 T。**

综合题：pending::<()>() 中两个 () 各做什么？
