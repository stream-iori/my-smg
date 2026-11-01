# 扩展：clap 的参数派生

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**当前项目手动解析命令行；clap 是可选的命令行解析方案，不属于现有依赖。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/bin/fake-worker.rs](../../../src/bin/fake-worker.rs)：`env::args` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## clap：`Parser`、`Args`、`Subcommand`、`ValueEnum`

```rust
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value_t = 3000)]
    port: u16,

    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,
}
```

| Derive | 用途 |
|---|---|
| `Parser` | 定义顶层命令行解析器 |
| `Args` | 定义一组可复用参数 |
| `Subcommand` | 用 enum 定义子命令 |
| `ValueEnum` | 将有限枚举映射为命令行参数值 |

SGL Model Gateway 的命令行入口同时使用了这些派生，见
[`main.rs`](../../../../sgl-model-gateway/src/main.rs)。

## 自查与关联

派生参数解析和业务校验有什么不同？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
