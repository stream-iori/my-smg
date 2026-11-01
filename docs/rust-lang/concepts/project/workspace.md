# Workspace：统一管理多个包

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**workspace 统一构建与公共配置，不把成员源码合并成一个 crate。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [Cargo.toml](../../../../Cargo.toml)：`[package]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先用 Maven 对照理解

如果你熟悉 Maven 多模块工程，可以先按下面的关系理解：

| Cargo / Rust 概念 | Maven 中相近的概念 | 含义与区别 |
|---|---|---|
| Cargo | Maven | 管理依赖、构建和测试的工具 |
| `Cargo.toml` | `pom.xml` | 描述工程的元数据、依赖和构建设置 |
| workspace | 多模块聚合工程 | 把多个包放在一起管理和构建 |
| `[workspace].members` | 聚合 POM 的 `<modules>` | 列出加入聚合工程的成员路径 |
| member package | Maven 子模块工程 | 有自己的清单；member 是“成员”身份，package 是“包”本身 |
| crate | 没有直接对应项 | Rust 的编译单元；不能简单等同于 Maven 模块或 JAR |
| Rust `mod` | 粗略类似 Java package 的代码组织作用 | 在 crate 内组织代码；不是 Maven module，也不是 Cargo package |

```text
Cargo                              Maven 多模块工程
workspace/                         aggregator/
├─ Cargo.toml（members）            ├─ pom.xml（modules）
├─ service/                        ├─ service/
│  └─ Cargo.toml                   │  └─ pom.xml
└─ tools/                          └─ tools/
   └─ Cargo.toml                      └─ pom.xml
   ↑ member package                   ↑ Maven 子模块工程
```

注意 Maven 的“聚合”和“继承”是两回事：`<modules>` 决定一起构建哪些工程，
子工程的 `<parent>` 决定继承哪个 POM 的配置；聚合 POM 和父 POM 可以是同一个，也可以不同。
因此 Cargo Workspace 首先类比 Maven 的聚合工程，公共配置的继承要另外看。
参见 [Maven：POM、聚合与继承](https://maven.apache.org/guides/introduction/introduction-to-the-pom.html)。

## 为什么使用 Workspace

多个相关 package 放进同一个 Workspace 后，可以共享：

- 一份 `Cargo.lock`，让整个仓库使用一致的依赖解析结果；
- 一个 `target/` 目录，避免每个 package 重复保存构建产物；
- package 元数据和依赖版本等公共配置；
- 根目录中的构建 profile；
- 一组面向全部或指定 member 的 Cargo 命令。

与 Maven 的区别：Maven 各子模块通常有自己的 `target/`；Cargo Workspace 默认共享一个
`target/` 和一份 `Cargo.lock`。不要把 Maven 的构建目录习惯直接套过来。

```text
rust/
├─ Cargo.toml       <- workspace root manifest
├─ Cargo.lock       <- 整个 workspace 共用
├─ target/          <- 整个 workspace 共用
├─ sglang-grpc/
│  └─ Cargo.toml    <- member package
├─ sglang-mm/
│  └─ Cargo.toml    <- member package
└─ sglang-server/
   └─ Cargo.toml    <- member package
```

## 根 `Cargo.toml`

当前 SGLang Rust 目录使用的是 virtual workspace：根清单只有 `[workspace]`，没有
`[package]`。

可以类比 Maven 中只负责聚合、使用 `pom` packaging 的根工程。不过 Cargo 的 virtual
workspace 根本身不是 package；Maven 的聚合根仍是一个 POM 工程。

```toml
[workspace]
resolver = "3"
members = [
    "sglang-grpc",
    "sglang-mm",
    "sglang-server",
]
```

源码见 [`rust/Cargo.toml`](../../../../../rust/Cargo.toml)。

```text
有 [workspace]，没有 [package] -> virtual workspace
有 [workspace]，也有 [package] -> 根 package 本身也是一个 member
```

如果根目录自身也是 package，通常可以省略把 `"."` 写进 `members`；根 package 自动属于
Workspace。`members` 支持 glob，例如 `"crates/*"`，不应加入的目录可放进 `exclude`。

```toml
[workspace]
members = ["crates/*"]
exclude = ["crates/experimental"]
```

## 自查与关联

当前 my-smg 是 workspace 成员集合还是一个 package？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
