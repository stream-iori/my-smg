# 工程与模块

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) | 为什么 main.rs 不能访问库的 pub(crate) 项？ | 无；可以直接阅读。 | [Cargo.toml](../../../../Cargo.toml)：`[package]` |
| [模块声明如何找到文件](modules-and-files.md) | circuit_breaker.rs 与 circuit_breaker/ 是两个模块吗？ | [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md)基础部分；项目应用另按节标题补前置。 | [src/circuit_breaker.rs](../../../../src/circuit_breaker.rs)：`pub mod gate` |
| [crate、self、super 与 use](paths-and-imports.md) | super 指父类还是父模块？ | [模块声明如何找到文件](modules-and-files.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`use super::is_retryable_status` |
| [pub 与受限可见性](visibility.md) | 祖父模块能否访问 pub(super) 的函数？ | [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`pub(super) struct GatewayState` |
| [pub use：内部组织与公开入口](reexports.md) | 公开入口变短是否意味着目录变浅？ | [crate、self、super 与 use](paths-and-imports.md) · [pub 与受限可见性](visibility.md)基础部分；项目应用另按节标题补前置。 | [src/lib.rs](../../../../src/lib.rs)：`pub mod worker` |
| [edition：语言规则与编译器版本](edition.md) | edition、rust-version 与 package version 分别管什么？ | 无；可以直接阅读。 | [Cargo.toml](../../../../Cargo.toml)：`edition = "2024"` |
| [Workspace：统一管理多个包（可选）](workspace.md) | 当前 my-smg 是 workspace 成员集合还是一个 package？ | [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md)基础部分；项目应用另按节标题补前置。 | [Cargo.toml](../../../../Cargo.toml)：`[package]` |
| [Workspace 的继承、依赖与构建选择（可选）](workspace-configuration.md) | workspace.dependencies 是否自动添加依赖？ | [Workspace：统一管理多个包](workspace.md)基础部分；项目应用另按节标题补前置。 | [Cargo.toml](../../../../Cargo.toml)：`[dependencies]` |
| [Workspace 命令：选择构建哪些成员（可选）](workspace-commands.md) | -p 接受目录名还是 package.name？ | [Workspace 的继承、依赖与构建选择](workspace-configuration.md)基础部分；项目应用另按节标题补前置。 | [Cargo.toml](../../../../Cargo.toml)：`package 名与构建目标` |
