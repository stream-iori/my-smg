# Package 与 Crate：一个工程里有几棵编译树

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**package 管版本和构建目标，crate 是独立编译单元；同一个 package 不等于同一个权限范围。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | 无；可以直接阅读。 |
| 源码入口 | [Cargo.toml](../../../../Cargo.toml)：`[package]` |
| 阅读时机 | T00 读到基础自查；Workspace、Maven、JPMS 为可选对照。 |


## T00：最少需要理解的内容

| 名称 | 作用 |
|---|---|
| Cargo package | 清单管理的构建、版本与依赖单位 |
| library crate | 通常从 lib.rs 开始，提供可复用接口 |
| binary crate | 通常从 main.rs 开始，提供程序入口 |

同一个 package 的库与二进制不是同一个 crate，后者不能直接访问库的 pub(crate) 项。
cargo check 检查编译与类型，不启动程序；cargo run 运行指定二进制；cargo test 执行已有测试。
测试函数用 #[test] 标记，测试专用模块可用 #[cfg(test)]；assert_eq 比较实际结果与预期值。
测试场景由学习者编写，命令成功不等于已经验证边界。当前网关入口还需要配置参数。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | lib.rs 与 main.rs 同时存在时，至少有几个 crate？ |
| 解释 | 为什么同包不能绕过 pub(crate)？ |
| 边界 | cargo check 成功是否证明运行行为正确？ |

先写下判断和原因，再用自己的已有代码验证。

## 可选：完整布局与 Java 对照

T00 到基础自查即可；下面的 Workspace、JPMS 和假定布局不作为前置。Config、Gateway 等示意名字不等于当前项目的类型清单。

## Workspace、Package 与 Crate 的层级

典型关系是：

```text
Workspace
├─ Package A（Cargo.toml）
│  ├─ library crate（src/lib.rs）
│  └─ binary crate（src/bin/tool.rs）
└─ Package B（Cargo.toml）
   └─ binary crate（src/main.rs）
```

Workspace 不是把所有源码合并成一个 crate。每个 member 仍是独立 package，拥有自己的依赖、
feature 和编译目标；Workspace 只是让 Cargo 能统一管理它们。

对应 Maven：聚合工程列出多个子模块，不会把它们的源码自动合并成一个编译单元。
Cargo 的一个 package 可以包含 library 和多个 binary crate，所以 package 与 crate 也不能等同。

## Cargo 如何识别 library 和 binary

结论：Cargo 主要根据文件位置或 `Cargo.toml` 中的 target 声明识别 binary，不是看到
`fn main()` 才把文件认作 binary。

```text
文件位置 / Cargo.toml 声明
          │
          v
Cargo 识别构建 target
          │
          v
Rust 编译器检查 binary 是否有 fn main()
```

默认目录约定：

| 文件 | Cargo 识别出的 target |
|---|---|
| `src/lib.rs` | library，默认名称来自 package 名 |
| `src/main.rs` | 默认 binary，默认名称来自 package 名 |
| `src/bin/fake-worker.rs` | 名为 `fake-worker` 的额外 binary |
| `src/bin/admin.rs` | 名为 `admin` 的额外 binary |

例如：

```text
my-smg/
├─ Cargo.toml
└─ src/
   ├─ lib.rs              -> library crate
   ├─ main.rs             -> binary crate：my-smg
   └─ bin/
      └─ fake-worker.rs   -> binary crate：fake-worker
```

运行指定 binary：

```bash
cargo run --bin my-smg
cargo run --bin fake-worker
```

如果 `src/bin/fake-worker.rs` 存在但没有 `fn main()`：

```text
Cargo：已经识别出 fake-worker binary target
编译器：报错，binary crate 缺少 main 函数
```

所以两者职责不同：

| 机制 | 回答的问题 |
|---|---|
| Cargo target 发现规则 | 哪些源码需要作为 library 或 binary 构建？ |
| `fn main()` | binary 从哪里开始执行？ |

也可以不使用默认目录，通过 `Cargo.toml` 显式声明：

```toml
[[bin]]
name = "fake-worker"
path = "src/other/worker.rs"
```

这时 `path` 决定入口文件，`name` 决定 `cargo run --bin` 后使用的名称。

## `crate`：像 JPMS module 一样理解边界

Java 的一个 JPMS module 可以包含多个 package；Rust 的一个 crate 可以包含多个 mod。两者都让你区分“内部实现”和“对外接口”。

但 Cargo package 和 crate 还要分开：**package 是 Cargo 管理构建、版本和发布的单位；crate 是 Rust 编译的单位。**

| 概念 | 管什么 | Java 对照 |
|---|---|---|
| Cargo package | 名字、版本、依赖和构建目标，由 `Cargo.toml` 描述 | 接近带 `pom.xml` 的 Maven 工程模块 |
| crate | 从一个根模块出发，独立编译一棵模块树 | 封装边界接近 JPMS module |
| `mod` | crate 内部组织类型、函数等项 | 接近 Java package |

```text
一个 Cargo package
├── src/lib.rs  → library crate 的根模块
└── src/main.rs → binary crate 的根模块
```

一个 package 可以包含至多一个 library crate，以及多个 binary crate。`lib.rs` 与 `main.rs` 即使在同一工程里，也属于两个 crate。[Rust：Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)

| 路径所在位置 | `crate::` 从哪里开始找 |
|---|---|
| `lib.rs` 及其子模块 | library crate 的根模块 |
| `main.rs` 及其子模块 | binary crate 的根模块 |

**`crate::` 指当前正在编译的 crate，不是整个 Cargo package。** 因此，库中的 `pub(crate)` 项不能仅因为 `main.rs` 在同一个 package 中，就被 binary crate 访问。

## 先看文件，再数 package 和 crate

```text
gateway-demo/                      ← 1 个 Cargo package
├── Cargo.toml                     ← 名字、版本、依赖、构建目标
└── src/
    ├── lib.rs                     ← library crate 的根
    ├── api.rs                     ← 库的 api 模块
    ├── internal/
    │   ├── mod.rs                 ← 库的 internal 模块
    │   └── routing.rs             ← internal 的子模块
    ├── main.rs                    ← 第一个 binary crate 的根
    └── bin/
        └── inspect.rs             ← 第二个 binary crate 的根
```

| 数量 | 对象 | 职责 |
|---|---|---|
| 1 个 package | `gateway-demo` | 一起管理版本、依赖和发布 |
| 1 个 library crate | 根是 `lib.rs` | 提供公开 API 和内部实现 |
| 2 个 binary crate | 根分别是 `main.rs`、`inspect.rs` | 分别提供启动程序和检查程序 |

所以这里共 **3 个 crate**。`api.rs`、`routing.rs` 是库的模块文件，不因为独立成文件就成为新 crate。假设采用默认库名，Rust 中用 `gateway_demo` 引用这个库：package 名中的连字符在默认库名中变成下划线。

## 自查与关联

为什么 main.rs 不能访问库的 pub(crate) 项？

[模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md) · [pub 与受限可见性](visibility.md)
