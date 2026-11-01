# Workspace 的继承、依赖与构建选择

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**根清单提供公共设置，成员显式取用；成员身份不等于自动依赖其他成员。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Workspace：统一管理多个包](workspace.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [Cargo.toml](../../../../Cargo.toml)：`[dependencies]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 统一 package 元数据

Workspace 根可以集中声明公共元数据：

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"
```

member 不会自动继承这些值，必须显式选择继承：

```toml
[package]
name = "sglang-mm"
version.workspace = true
edition.workspace = true
license.workspace = true
```

类型变化可以这样理解：

```text
[workspace.package].edition = "2024"   <- 定义公共值
edition.workspace = true               <- member 显式取用
```

因此在根清单中声明一个字段，不等于强制覆盖所有 member。

Maven 类比：父 POM 可以集中提供公共配置。区别是 Cargo 成员必须逐字段写
`.workspace = true` 才取用；不能按 Maven 的继承习惯认为这些值会自动进入成员包。

## 统一依赖版本

根清单可以作为依赖配置的单一来源：

```toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

需要这些依赖的 member 再显式继承：

```toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
```

这里有两个关键点：

1. `[workspace.dependencies]` 不会自动把依赖添加到每个 member；
2. member 可以在继承时追加 `features`，但不能改成另一个 `version`。

Maven 类比：`[workspace.dependencies]` 接近父 POM 的 `<dependencyManagement>`：
集中管理依赖配置，成员还要在自己的依赖列表中声明使用它。
它不是 Maven 父 POM 中会被子工程继承的普通 `<dependencies>`。
另外，Maven 子工程可以在自己的依赖声明中覆盖管理的版本；Cargo 成员使用
`workspace = true` 时不能同时指定自己的版本。
参见 [Maven：依赖管理机制](https://maven.apache.org/guides/introduction/introduction-to-dependency-mechanism.html)。

```toml
[dependencies]
serde = { workspace = true, features = ["rc"] }
```

最终启用的 feature 还会受到 Cargo feature 合并规则影响，所以“某个依赖在一个 member 中
关闭默认 feature”不代表同一次 Workspace 构建中它一定不会被其他 member 打开。

## member 之间如何依赖

属于同一 Workspace，不代表 package 之间自动可见。`sglang-server` 要使用 `sglang-mm`，仍然
必须正常声明依赖：

对应 Maven：两个工程都列在 `<modules>` 中，仍需要在使用方的 `<dependencies>` 中声明
对另一个工程的依赖。Cargo 本地依赖可以用 `path` 找到包；Maven 通常使用
`groupId`、`artifactId`、`version` 坐标引用。

```toml
[dependencies]
sglang_mm = {
    package = "sglang-mm",
    path = "../sglang-mm",
    default-features = false,
}
```

上例中三个名字的含义不同：

| 名字 | 含义 |
|---|---|
| `sglang-mm` | 被依赖 package 的名字，即 `[package].name` |
| `../sglang-mm` | 相对于当前 `Cargo.toml` 的本地路径 |
| `sglang_mm` | 当前 crate 在 Rust 代码中使用的依赖名 |

```rust
use sglang_mm::processor::Processor;
```

即使不显式重命名，Cargo 也会把 package 名中的 `-` 转成 Rust 路径中的 `_`；显式写出别名
通常是为了让意图更清楚。

## `resolver` 是什么

`resolver` 选择 Cargo 的依赖与 feature 解析规则。当前 Workspace 使用：

```toml
[workspace]
resolver = "3"
```

它与 Rust 2024 edition 配套。尤其是 virtual workspace 没有根 package，Cargo 无法从
`[package].edition` 推断 resolver，因此应在 `[workspace]` 中显式声明。

要注意：`resolver = "3"` 不等于“不同 member 永远得到完全隔离的 feature 集合”。同一次
构建选择了哪些 package、target 和 feature，仍会影响最终解析结果。排查时先用精确的 package
命令复现，不要只看某个 `Cargo.toml` 作结论。

## profile 只能在根清单生效

构建 profile 应写在 Workspace 根：

```toml
[profile.release]
lto = true
strip = true
opt-level = 3
codegen-units = 1
```

member 清单里的 `[profile.dev]`、`[profile.release]` 会被忽略。若只想调整某个 package，
仍然在根清单写 package override：

```toml
[profile.release.package.sglang-server]
opt-level = 2
```

## 常见误区

| 误区 | 实际情况 |
|---|---|
| Workspace 就是一个大 crate | Workspace 管理多个 package；每个 target 仍单独编译成 crate |
| member 可以直接引用另一个 member | 仍需在 `[dependencies]` 中声明 `path` 或版本依赖 |
| 根依赖会自动进入全部 member | member 必须写 `{ workspace = true }` |
| 每个 member 有自己的 lockfile 和 `target/` | Workspace 默认共用根 `Cargo.lock` 和根 `target/` |
| 在 member 中写 profile 能单独生效 | profile 只读取 Workspace 根清单中的配置 |
| `cargo check` 永远检查整个 Workspace | 取决于执行位置、根清单类型和 `default-members` |
| package、crate、module 是同一层概念 | package 是 Cargo 单元，crate 是编译单元，module 是 crate 内代码组织单元 |
| Rust module 就是 Maven module | Rust module 在 crate 内组织代码；Maven module 更接近 Workspace 的 member package |
| Cargo 因为看见 `fn main()` 才识别 binary | Cargo 根据目录约定或 `[[bin]]` 识别 target；编译器再检查 `main` |

## 自查与关联

workspace.dependencies 是否自动添加依赖？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
