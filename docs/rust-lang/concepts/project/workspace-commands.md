# Workspace 命令：选择构建哪些成员

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**默认成员、整个工作区与指定包是不同构建范围，命令选择不等于依赖声明。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Workspace 的继承、依赖与构建选择](workspace-configuration.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [Cargo.toml](../../../../Cargo.toml)：`package 名与构建目标` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 常用命令

在 Workspace 根目录执行：

```bash
# 检查默认选择的 member
cargo check

# 检查所有 member
cargo check --workspace

# 只检查一个 package；-p 是 --package 的缩写
cargo check -p sglang-server

# 检查全部 member，但排除一个
cargo check --workspace --exclude sglang-grpc

# 对指定 package 运行测试
cargo test -p sglang-mm

# 查看 Cargo 解析出的 Workspace、target、依赖和 feature
cargo metadata --no-deps --format-version 1
```

不在 Workspace 根目录时，可以明确指定清单。下面的路径假设命令从 SGLang 仓库根目录执行：

```bash
cargo check \
  --manifest-path rust/Cargo.toml \
  --workspace
```

`-p` 接受的是 `[package].name`，不是目录名、`[lib].name` 或 Python 模块名。

## 用 Maven 命令帮助记忆

下面只对照“选择哪些工程”的作用，不表示编译、测试生命周期完全相同：

| Cargo 命令 | Maven 中相近的用法 | 选择范围 |
|---|---|---|
| `cargo check --workspace` | 聚合根执行 `mvn compile` | 全部成员；Cargo check 检查编译但不生成最终可执行文件 |
| `cargo check -p sglang-server` | `mvn -pl service compile` | 选择一个包/子模块；两者选择依赖一起构建的规则不同 |
| `cargo test --workspace` | 聚合根执行 `mvn test` | 对全部成员运行测试 |

Cargo 会构建所选包需要的依赖；Maven 需要同时构建所选模块依赖的 reactor 工程时，
通常加 `-am`。选择同一个聚合工程中的模块，不等于自动选择它的依赖模块。
参见 [Maven：多模块构建与 reactor 命令选项](https://maven.apache.org/guides/mini/guide-multiple-modules.html)。

## 默认构建哪些 member

可以用 `default-members` 控制在根目录运行不带 `-p`、`--workspace` 的命令时默认选择谁：

```toml
[workspace]
members = ["service", "tools/*"]
default-members = ["service"]
```

```text
cargo check              -> default-members
cargo check --workspace  -> members 中的全部 package
cargo check -p tool      -> 指定 package
```

对于没有设置 `default-members` 的 virtual workspace，根目录命令默认选择全部 member。

## 如何确认某个 package 属于哪个 Workspace

不要只根据目录层级猜测，直接让 Cargo 给出答案。下面的路径假设命令从 SGLang 仓库根目录
执行：

```bash
cargo metadata \
  --manifest-path rust/Cargo.toml \
  --no-deps \
  --format-version 1
```

重点看：

- `workspace_root`：Workspace 根目录；
- `workspace_members`：所有 member 的 package ID；
- `packages[].manifest_path`：每个 package 对应的清单；
- `target_directory`：共享构建目录。

当前 SGLang 的结果可以简化为：

```text
workspace_root: rust/
target_directory: rust/target/
members:
  - sglang-grpc
  - sglang-mm
  - sglang-server
```

## 自查与关联

-p 接受目录名还是 package.name？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
