# edition：语言规则与编译器版本

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**edition 选择语言兼容规则，不是选择 rustc 版本，也不会自动升级依赖。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | 无；可以直接阅读。 |
| 源码入口 | [Cargo.toml](../../../../Cargo.toml)：`edition = "2024"` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 当前项目的配置

```toml
[package]
name = "my-smg"
version = "0.1.0"
edition = "2024"
```

| 项 | 负责什么 |
|---|---|
| edition | 选择语言兼容规则，影响该包的各编译目标 |
| rustc 版本 | 实际使用的编译器实现 |
| rust-version | 项目声明支持的最低工具链版本 |
| version | 项目自己的发布版本 |

同一个新编译器可以按不同 edition 编译不同包，不同 edition 的依赖也可以互相使用。
Rust 2024 从 Rust 1.85.0 开始支持；截至本次整理，最新稳定 edition 是 2024。

```text
较新 rustc
├── edition 2021 包 → 2021 语言规则
└── edition 2024 包 → 2024 语言规则
```

从 2021 切换 2024 可能遇到 gen 保留关键字、模式匹配规则和临时值作用域等变化，需要迁移检查，不能只改一个数字就宣称通过。
更新 edition 不等于程序自动加速，也不等于更新依赖。学习项目只解释迁移流程，不由助手改配置或运行会重写源码的 cargo fix。

依据：[Cargo edition](https://doc.rust-lang.org/cargo/reference/manifest.html#the-edition-field)、[Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)。

## 自查与关联

edition、rust-version 与 package version 分别管什么？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
