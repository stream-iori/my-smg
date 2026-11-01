# 扩展：thiserror 的错误派生

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**当前项目手写 Error、Display 和 From；thiserror 可生成重复样板，但不是 ? 的前提。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) · [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) · [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../src/config.rs)：`ConfigLoadError` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## thiserror：`Error`

`thiserror` 没有发明新的错误体系，它只是自动生成标准库要求的 trait 实现。下面两种写法
功能相同。

如果还不熟悉 `fmt::Formatter<'_>`、生命周期、`Display`、`Error::source()` 或 `dyn Error`，先读
[`Formatter<'_>`、生命周期、`Display` 与 `Error`](../concepts/errors/debug-display-and-formatter.md)。

## 只使用 Rust 标准库

```rust
use std::{error::Error, fmt, io};

#[derive(Debug)]
enum CaptureError {
    InvalidStatus(u64),
    Repository(io::Error),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStatus(payment_id) => {
                write!(f, "payment {payment_id} has invalid status")
            }
            Self::Repository(_) => write!(f, "repository operation failed"),
        }
    }
}

impl Error for CaptureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::InvalidStatus(_) => None,
        }
    }
}

impl From<io::Error> for CaptureError {
    fn from(error: io::Error) -> Self {
        Self::Repository(error)
    }
}
```

除了错误 enum，标准库写法还要手写：

```text
Display      -> 错误显示什么文字
Error        -> 哪个字段是下层错误 source
From         -> ? 如何把下层错误转换成当前错误
```

## 使用 `thiserror` 的等价写法

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum CaptureError {
    #[error("payment {0} has invalid status")]
    InvalidStatus(u64),

    #[error("repository operation failed")]
    Repository(#[from] std::io::Error),
}
```

两段代码的对照关系是：

| 标准库手写 | `thiserror` |
|---|---|
| `impl fmt::Display` | `#[error("...")]` |
| `impl Error { source(...) }` | 自动把 `#[from]` 字段作为 source |
| `impl From<io::Error>` | `#[from]` |

两种写法都能让 `?` 自动转换错误：

```rust
fn query_payment() -> Result<(), std::io::Error> {
    Err(std::io::Error::other("database unavailable"))
}

fn capture_payment() -> Result<(), CaptureError> {
    query_payment()?; // io::Error -> CaptureError::Repository
    Ok(())
}
```

这行 `?` 近似于：

```rust
query_payment().map_err(CaptureError::Repository)?;
```

因此 `thiserror` 的优势不是功能更多，而是把三个实现压缩成两个属性：`#[error("...")]` 生成
`Display`，`#[from]` 生成错误转换并把内部错误标记为 source。新增 variant 时不用同步维护多段
`match`。

`thiserror` 在编译期生成的仍然是标准 `std::error::Error` 实现，调用方不需要依赖或感知
`thiserror`。它不会生成 `Debug`，所以通常同时写 `#[derive(Debug, Error)]`。实际用法见
[`core/error.rs`](../../../../sgl-model-gateway/src/core/error.rs) 和
[`utils/error.rs`](../../../../rust/sglang-server/src/utils/error.rs)。

## 自查与关联

thiserror 自动生成能力，是否等于自动处理错误？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
