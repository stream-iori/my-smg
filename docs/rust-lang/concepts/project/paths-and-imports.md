# crate、self、super 与 use

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**路径决定从哪里找，use 只是给已有名字提供短写法。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [模块声明如何找到文件](modules-and-files.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`use super::is_retryable_status` |
| 阅读时机 | T00 读到基础自查；完整布局和 Java 对照可跳过。 |


## T00：最少需要理解的内容

| 写法 | 起点或职责 |
|---|---|
| crate:: | 当前正在编译的 crate 根 |
| self:: | 当前模块 |
| super:: | 父模块 |
| use | 名字简写，不改变权限 |

库与二进制各有自己的 crate 根。self:: 与 super:: 不是 Java 对象与继承关系。
依赖由 Cargo.toml 配置，use 不下载依赖，也不建立模块。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | main.rs 中 crate:: 指库还是二进制？ |
| 解释 | use 能否绕过目标项的私有权限？ |
| 边界 | super:: 代表父类还是父模块？ |

先写下判断和原因，再用自己的已有代码验证。

## 可选：完整布局与 Java 对照

T00 到基础自查即可；下面的 Workspace、JPMS 和假定布局不作为前置。Config、Gateway 等示意名字不等于当前项目的类型清单。

## 模块路径：从哪里开始找

以 `crate::config::Config` 为例：

```text
当前 crate 根模块 → config 模块 → Config 类型
```

这是编译期名字解析，不是运行时读取文件，也不会创建对象。

| Rust 路径 | 起点或含义 | Java 对照 |
|---|---|---|
| `crate::config::Config` | 从当前 crate 根模块找 | 接近使用完整限定名，但 Java 类名通常不带 JPMS module 名 |
| `self::helper` | 从当前模块找 | 不是 Java 的 `this`，不表示当前对象 |
| `super::Router` | 从父模块找 | 不是 Java 的 `super`，不表示父类 |
| `依赖名::类型名` | 从外部依赖 crate 找 | 接近访问依赖模块所提供的类型 |

**Rust 的模块路径表达组织关系，不表达类的继承关系。**

## `use`：最接近 Java import

| 操作 | Java 9 | Rust |
|---|---|---|
| 写完整路径 | `com.example.config.Config` | `crate::config::Config` |
| 引入名字 | `import com.example.config.Config;` | `use crate::config::Config;` |
| 引入后引用类型 | `Config` | `Config` |

`import` 与 `use` 都让当前作用域少写路径。它们不创建对象、不复制代码，也不改变目标的访问权限。

```text
Java requires → 声明模块依赖/可读性
Java import   → 引入名字简写

Rust mod      → 声明模块
Rust use      → 引入名字简写
```

Java 的依赖声明与名字引入规则见 [Java 9 语言规范](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html)；Rust 导入路径同样需要通过 [可见性检查](https://doc.rust-lang.org/reference/visibility-and-privacy.html)。

## 同一个名字，从不同位置怎么找：路径与 `use`

| 使用位置 | 路径或名字 | 解释 |
|---|---|---|
| 库的 `internal::routing` 中 | `crate::api::Gateway` | 从 library crate 根找公开类型 |
| 库的 `internal::routing` 中 | `self::cursor` | 从当前 `routing` 模块找私有函数 |
| 库的 `internal::routing` 中 | `super::normalize` | 从父模块 `internal` 找私有函数；子模块有访问权限 |
| 库的 `api` 中 | `crate::internal::routing::choose` | 仍在同一 library crate，能访问 `pub(crate)` 函数 |
| 启动 binary 中 | `gateway_demo::api::Gateway` | 从依赖的 library crate 找类型 |
| 检查 binary 中 | `gateway_demo::api::Gateway` | 同样使用库的公开 API |
| 任一 binary 中 | `crate::api::Gateway` | 本例无此路径：`crate::` 指该 binary，它没有声明 `api` 模块 |

库的 `api` 模块若用 `use` 引入 `choose`，后续可以用短名 `choose`；binary 若用 `use` 引入 `Gateway`，后续可以用短名 `Gateway`。两者都只是简写，权限不变。

因此，binary 不能靠 `use` 引入库里的 `choose` 来绕过 `pub(crate)`。`Cargo.toml` 负责依赖配置，`use` 不负责下载或添加依赖。

## 自查与关联

super 指父类还是父模块？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [pub 与受限可见性](visibility.md)
