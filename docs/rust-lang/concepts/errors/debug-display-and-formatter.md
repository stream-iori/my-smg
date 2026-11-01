# Debug、Display 与 Formatter

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Debug 帮开发者观察结构，Display 面向阅读者；Formatter 是向格式化输出写入的上下文。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`impl Display for ConfigLoadError` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `Debug`：开发时查看值

```rust
#[derive(Debug)]
struct Request {
    id: u64,
    prompt: String,
}

let request = Request {
    id: 1,
    prompt: "hello".to_owned(),
};

println!("{request:?}");   // 紧凑格式
println!("{request:#?}");  // 美化格式
```

常用于日志、断言失败信息和临时排查。所有字段都必须实现 `Debug`。

`Debug` 面向开发者，不是稳定的用户协议；不要依赖其输出格式做解析，也不要无意间把 token、
密码等敏感字段写进日志。

## `fmt::Formatter<'_>` 拆开看

| 部分 | 含义 |
|---|---|
| `fmt` | `std::fmt` 模块的简称 |
| `Formatter` | 保存格式选项和输出目标的类型 |
| `<'_>` | 存在一个生命周期参数，由编译器推断 |
| `&mut Formatter<'_>` | 在本次 `fmt` 调用期间独占借用格式化器 |

标准库的真实实现细节可能随版本演进，但核心结构可以简化理解为：

```rust
pub struct Formatter<'a> {
    options: FormattingOptions,
    buf: &'a mut dyn Write,
}
```

其中：

```text
options -> 宽度、精度、对齐、正负号等格式选项
buf     -> 实际接收文本的输出缓冲区
```

用户通常不直接构造 `Formatter`。`println!`、`format!`、`write!` 等格式化系统创建它，再把
`&mut Formatter` 交给 `Display::fmt` 或 `Debug::fmt`。

## 为什么 Formatter 需要生命周期

`Formatter` 不拥有输出缓冲区，它只借用缓冲区：

```text
输出缓冲区（所有者）
      │
      │ &'a mut
      ▼
Formatter<'a>
```

因此编译器必须保证：

```text
Formatter 的使用时间 <= 被借用缓冲区的存活时间
```

如果缓冲区已经销毁，Formatter 仍然存在，它内部的指针就会悬空。生命周期参数让这种情况在
编译期被拒绝。

生命周期主要描述引用之间的有效范围关系，不是在运行时增加计时器，也不会让值活得更久。

## 这里其实有两层借用

```rust
f: &mut fmt::Formatter<'_>
```

概念上可以展开为：

```text
&'call mut Formatter<'buffer>
   │                    │
   │                    └─ Formatter 内部借用输出缓冲区的时间
   └─ fmt 方法临时借用 Formatter 的时间
```

通常满足：

```text
'call <= 'buffer
```

源码把外层 `&mut` 的生命周期省略，把内层 Formatter 的生命周期写成 `'_`，两者都由编译器
推断。

## 为什么是 `&mut Formatter`

写入会推进缓冲区状态：

```text
空缓冲区
   -> 写入 "duplicate worker id: "
   -> 写入 worker_id
   -> 格式化结束
```

所以格式化方法需要可变访问 Formatter。`&mut` 同时保证本次写入期间没有另一个写入者并发
修改同一个 Formatter。

它不表示 `fmt` 拥有 Formatter。调用结束后，格式化系统仍然拥有并继续管理它。

## `Display` 的真实职责

标准库 trait 的核心形状是：

```rust
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}
```

职责是：

```text
把 self 的人类可读表示写入 f
```

`Display` 不直接负责：

- 选择控制台还是字符串；
- 打开文件；
- 记录日志级别；
- 给错误加颜色；
- 决定进程退出码。

这些由调用方或更上层组件决定。

## 哪些操作会使用 Display

| 写法 | 使用的 trait | 输出目标 |
|---|---|---|
| `println!("{value}")` | `Display` | 标准输出 |
| `eprintln!("{value}")` | `Display` | 标准错误 |
| `format!("{value}")` | `Display` | 新建的 `String` |
| `value.to_string()` | `Display`，通过 `ToString` | 新建的 `String` |
| `println!("{value:?}")` | `Debug` | 标准输出 |
| `format!("{value:#?}")` | `Debug` | 新建的 `String` |

实现 `Display` 后，标准库为该类型提供 `ToString` 的通用实现。因此通常不需要自己实现
`ToString`。

## `Debug` 与 `Display`

```text
Debug
├─ 面向开发者
├─ 常展示 enum variant 和字段结构
└─ 可以 derive

Display
├─ 面向用户、CLI、日志摘要
├─ 由类型作者决定简洁消息
└─ 通常手写或由 thiserror 生成
```

以配置错误为例：

```text
Debug:
InvalidWorkerAddress("worker-a", "not-a-url")

Display:
invalid address for worker worker-a: not-a-url
```

`Display` 文本不应自动被当作稳定的机器协议。程序需要分类错误时，应匹配 enum variant，而
不是解析显示字符串。

## `write!` 为什么返回 `fmt::Result`

`fmt::Result` 是格式化模块使用的结果类型，概念上等价于：

```rust
Result<(), fmt::Error>
```

```text
Ok(())          文本写入成功，没有额外值需要返回
Err(fmt::Error) 输出目标拒绝或无法完成写入
```

`write!(f, "...")` 已经返回 `fmt::Result`，所以一个 match 分支可以直接把它作为结果返回。

格式化失败不是业务错误，因此 `fmt` 方法一般只传播 `fmt::Error`，不要在这里执行网络请求、
复杂校验或产生其他副作用。

## `Formatter` 缺少生命周期参数

`Formatter` 是带生命周期参数的类型。实现格式化 trait 时通常使用 `Formatter<'_>`。

## `ConfigError` 没有实现 Display

常见表现是不能使用 `{}` 或不能实现 `Error`。先实现 `Display`，或者使用 `thiserror` 生成。

## 自查与关联

为什么 fmt 接收 &mut Formatter<'_>？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
