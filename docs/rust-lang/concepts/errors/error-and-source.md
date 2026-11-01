# 错误枚举、Error 与 source

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**枚举表达错误分类，Error 提供统一接口，source 借用暴露底层原因。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Debug、Display 与 Formatter](debug-display-and-formatter.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`ConfigLoadError / Error::source` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `std::error::Error` 的核心结构

标准库中的关系可以简化为：

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

`Debug + Display` 是 supertrait 约束：一个类型要实现 `Error`，必须先能用两种方式显示。

```text
ConfigError
├─ Debug   开发者格式
├─ Display 用户可读格式
└─ Error   标准错误接口
```

## 空的 `impl Error` 为什么有意义

```rust
impl std::error::Error for ConfigError {}
```

虽然实现体为空，但它明确承诺：

- 该类型满足标准错误契约；
- 可以作为 `dyn Error` 使用；
- 可以交给要求 `E: Error` 的泛型 API；
- `source()` 使用默认实现，返回 `None`。

当前 `ConfigError` 的三个 variant 都直接保存业务信息，没有包裹底层错误，所以 `None` 合理。

以后如果增加：

```text
ConfigError::Json(serde_json::Error)
```

就可以让 `source()` 返回内部的 `serde_json::Error`，形成错误原因链。

## 实现 Error 不会自动做什么

实现 `Error` 不会自动：

- 打印到控制台；
- 写日志；
- panic；
- 重试；
- 给进程设置退出码；
- 把任意错误转换成 `ConfigError`；
- 让所有 `?` 自动通过。

它提供的是统一接口，不是错误处理策略。

## 泛型代码

```rust
fn report<E: std::error::Error>(error: &E) {
    eprintln!("{error}");
}
```

调用方可以传入不同具体错误，函数只依赖 `Error` 契约。

## 动态分发

```rust
fn report(error: &dyn std::error::Error) {
    eprintln!("{error}");
}
```

调用时通过 trait object 的虚表找到具体 `Display` / `Error` 实现。trait object 原理见
[Trait、静态分发、动态分发与智能指针](../types/traits-and-bounds.md)。

## 多种错误的统一返回

```rust
fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 可以向上传播多种满足要求的错误
    Ok(())
}
```

`Box` 拥有具体错误，`dyn Error` 擦除具体类型。适合应用边界或快速组合；领域库通常更适合返回
明确的错误 enum，让调用者可以匹配具体 variant。

## `From` 与 `source()` 不要混淆

结论：`From` 用在错误产生和向上传播时，`source()` 用在错误产生后追查原因时。

| 对比 | `From` | `source()` |
|---|---|---|
| 方向 | 底层错误 → 外层错误 | 外层错误 → 查看底层错误 |
| 所有权 | 移动并包装错误 | 只返回共享引用 |
| 主要调用者 | `?` 或显式 `ConfigLoadError::from(...)` | 错误报告器、日志或诊断代码 |
| 是否改变错误类型 | 是 | 否 |
| 没有底层原因时 | 不适用 | 返回 `None` |

以配置加载为例：

```text
错误发生时：From

serde_json::Error
        │ move
        ▼
ConfigLoadError::Parse(error)

错误报告时：source()

ConfigLoadError::Parse(error)
        │ borrow
        ▼
&serde_json::Error
```

转换实现：

```rust
impl From<serde_json::Error> for ConfigLoadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(error)
    }
}
```

原因链实现：

```rust
impl std::error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}
```

两者连接成完整流程：

```text
1. serde_json 产生错误
2. ? 使用 From，把它包装成 ConfigLoadError::Parse
3. 调用方收到 ConfigLoadError
4. 诊断代码通过 source() 查看原始 serde_json::Error
```

一句话：

```text
From 负责“装进去”，source() 负责“看里面”。
```

`?` 的完整控制流见[`Result`、`?` 与错误传播](question-mark.md)。

## `source()` 返回的引用生命周期不够长

检查内部错误是否借用了短期数据。不要通过泄漏内存或强行标记 `'static` 绕过；优先让错误
拥有必要的上下文数据。

## 错误类型不同时发生什么

文件读取可能返回 `std::io::Error`，JSON 解析可能返回 `serde_json::Error`，而函数也许统一返回
自定义的 `ConfigError`。

要让 `?` 自动传播，当前错误必须能够转换成函数声明的错误类型。概念上需要：

```text
ConfigError: From<std::io::Error>
ConfigError: From<serde_json::Error>
```

传播过程近似为：

```text
Err(serde_json::Error)
       │ From::from
       ▼
Err(ConfigError)
```

这就是 `thiserror` 中 `#[from]` 或手写 `From` 实现经常与 `?` 一起出现的原因。相关内容见
[常见 derive 速查中的 thiserror 章节](../../advanced/thiserror.md)。

`Error`、`Display`、`From` 和 `?` 各自负责什么，以及 `dyn Error` 如何统一不同错误，见
[`Formatter<'_>`、生命周期、`Display` 与 `Error`](debug-display-and-formatter.md)。
其中 `From` 与 `source()` 的方向和所有权差异，见该文的
[`From` 与 `source()` 不要混淆](error-and-source.md)。

## 自查与关联

From 会拿走错误，source 会拿走吗？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
