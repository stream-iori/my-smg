# 模块声明如何找到文件

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**mod 建立模块树；目录存在不等于模块已经声明。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/circuit_breaker.rs](../../../../src/circuit_breaker.rs)：`pub mod gate` |
| 阅读时机 | T00 读到基础自查；多层模块按需回查。 |


## T00：最少需要理解的内容

| 项目项 | 作用 |
|---|---|
| lib.rs | 库的根模块 |
| pub mod worker; | 声明并公开子模块 |
| worker.rs | 默认布局下该子模块的内容 |
| use | 引入已有名字，不代替模块声明 |

文件存在不等于已经声明模块。公开类型还要检查沿途模块的可见性。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 只新建 rs 文件会自动得到子模块吗？ |
| 解释 | mod 与 use 为什么不能互相代替？ |
| 边界 | pub mod 改成 mod 后，寻找文件的位置变了吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 可选：完整布局与 Java 对照

T00 到基础自查即可；下面的 Workspace、JPMS 和假定布局不作为前置。Config、Gateway 等示意名字不等于当前项目的类型清单。

## 从 `pub mod gate;` 到 `circuit_breaker/gate.rs`：逐层找文件

**不是文件夹自动关联，而是 `mod` 声明让编译器按约定寻找模块内容。**
下面采用默认文件布局，不使用 `#[path]` 改写位置。

| 声明写在哪里 | 声明内容 | 编译器寻找的文件 |
|---|---|---|
| `src/lib.rs` | `pub mod circuit_breaker;` | `src/circuit_breaker.rs` |
| `src/circuit_breaker.rs` | `pub mod gate;` | `src/circuit_breaker/gate.rs` |

上表展示当前选用的路径。第一行也可以用 `src/circuit_breaker/mod.rs`，
第二行也可以用 `src/circuit_breaker/gate/mod.rs`；同一模块的两种候选文件不能同时存在。

```text
src/lib.rs
  │ pub mod circuit_breaker;
  ▼
src/circuit_breaker.rs
  │ pub mod gate;
  ▼
src/circuit_breaker/gate.rs
  └─ 定义 CircuitGate 等类型
```

这里 `circuit_breaker.rs` 与同名目录 `circuit_breaker/` 可以同时存在：

```text
src/
├── lib.rs
├── circuit_breaker.rs       ← 父模块自己的内容，以及子模块声明
└── circuit_breaker/
    └── gate.rs              ← 子模块的内容
```

**`circuit_breaker.rs` 与 `circuit_breaker/` 不是两个 circuit_breaker 模块。**

| 文件或目录 | 承担什么 |
|---|---|
| `circuit_breaker.rs` | circuit_breaker 模块自己的定义，以及 `pub mod gate;` 声明 |
| `circuit_breaker/` | 按默认约定，存放该模块的子模块文件；目录本身不额外创建模块 |
| `circuit_breaker/gate.rs` | gate 子模块自己的定义 |

从“磁盘上的文件”换到“编译器里的模块树”，是下面的关系：

```text
crate（lib.rs）
└── circuit_breaker（circuit_breaker.rs）
    ├── CircuitBreaker：结构体
    ├── CircuitState：枚举
    └── gate（circuit_breaker/gate.rs）
        └── CircuitGate：结构体
```

先有 `mod` 声明建立父子关系，再按默认文件规则寻找定义，不是看到同名文件夹就自动扫描全部文件。

对应的模块路径是 `my_smg::circuit_breaker::gate::CircuitGate`。
文件层次用于定位源码，模块路径用于编译期解析名字，不是运行时加载文件。

## 把 mod、pub、use 分开看

| 写法 | 负责什么 | 不负责什么 |
|---|---|---|
| `mod gate;` | 声明子模块，从文件加载定义 | 不自动向外公开模块 |
| `pub mod gate;` | 声明子模块，并开放模块可见性 | 不自动公开内部所有类型和字段 |
| `use gate::CircuitGate;` | 引入已有名字，后面可以写短名 | 不代替 mod 声明，也不绕过可见性 |

所以，将 `pub mod gate;` 改成 `mod gate;`，**寻找的文件不变，只是访问权限变了**。
外部能使用完整路径，还需要库根公开 circuit_breaker、父模块公开 gate，以及 CircuitGate 本身公开。

在 `gate.rs` 中写：

```rust
use super::{CircuitBreaker, CircuitState};
```

`super` 指模块树上的父模块 circuit_breaker，因此能找到父模块定义的类型。
不是“扫描上一级文件夹”，也不是 Java 的父类。

这里也可以写成完整路径，两种写法引入的是同样的类型：

```rust
use crate::circuit_breaker::{CircuitBreaker, CircuitState};
```

编译器先根据 `mod` 声明建立模块树，再沿 `super` 或 `crate::` 查找名字。
不是 `use` 在运行时打开 circuit_breaker.rs，也没有创建另一份 CircuitBreaker。

## 与 mod.rs 写法的关系

| 模块 | 当前文件写法 | 另一种合法写法 |
|---|---|---|
| circuit_breaker | `src/circuit_breaker.rs` | `src/circuit_breaker/mod.rs` |
| circuit_breaker::gate | `src/circuit_breaker/gate.rs` | `src/circuit_breaker/gate/mod.rs` |

同一个模块只能选其中一种，不能同时用两份候选文件定义它，否则会产生歧义。
两种布局不改变模块路径，也不改变访问权限。

本项目 `gateway.rs` 中的 `mod retry;`、`#[cfg(test)] mod tests;` 遵循同样规则，
分别定位 `gateway/retry.rs` 和 `gateway/tests.rs`。tests 这个名字本身没有特殊加载规则，
`#[cfg(test)]` 才决定该模块只在测试构建中加入。

一句话：**`mod` 连上模块树，约定决定文件位置，`pub` 决定权限，`use` 简写名字。**
另见 [Cargo 如何识别 library 和 binary](package-and-crate.md)：crate 根文件的发现是另一层规则。

Java 包名与 Rust 模块树最大的差别是：

| 情况 | Java 9 | Rust |
|---|---|---|
| 子包/子模块访问父级私有内容 | 子包没有特殊权限 | 子模块可以访问父模块的私有项 |
| 名称层级 | `com.example` 与 `com.example.config` 是不同的包 | `config` 可以是根模块的实际子模块 |
| 访问控制 | 包名层级不赋予额外权限 | 模块父子关系参与权限判断 |

Java 的包名可以有层级，但子包不会继承父包的包访问权限。[Java 9：Package Members](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html#jls-7.1)

## 文件如何进入模块树：`mod` 与 `pub mod`

假设库根声明公开的 `api` 模块和私有的 `internal` 模块；`internal` 再声明仅 crate 内可见的 `routing` 子模块。两个 binary 不重复声明这些库模块，而是使用库的公开入口。

```text
library crate：gateway_demo（lib.rs）
├── api                            ← pub mod：公开路径
│   └── Gateway                    ← pub 类型
└── internal                       ← mod：对外隐藏
    ├── normalize                  ← 私有函数
    └── routing                    ← pub(crate) mod：仅库内部
        ├── choose                 ← pub(crate) 函数
        ├── reset                  ← pub(super) 函数
        └── cursor                 ← 私有函数

binary crate：启动程序（main.rs）
└── main

binary crate：检查程序（inspect.rs）
└── main
```

这三棵树分别编译。`mod` 负责声明树上的节点；`pub mod` 还开放模块的可见性。`use` 只引入已有名字，不建立这些模块。

## `mod`：接近 Java package，但父子关系有实际作用

假设目录如下：

```text
src/
├── lib.rs
├── config.rs
└── routers/
    ├── mod.rs
    └── http.rs
```

只有声明了对应模块，才形成这棵树：根模块声明 `config` 和 `routers`，`routers` 再声明 `http`。

```text
crate（根模块：lib.rs）
├── config
└── routers
    └── http
```

| Rust 写法 | 意思 |
|---|---|
| `mod config;` | 声明 config 模块，默认私有 |
| `pub mod config;` | 声明 config 模块，并将它标记为公开；外部访问仍须有可访问路径 |

文件形式的模块通常对应 `config.rs` 或 `config/mod.rs`，也可以把模块内容写在声明处。**文件存在不代表自动加入模块树，`use` 也不能代替模块声明。** [Rust：模块组织](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)

## 自查与关联

circuit_breaker.rs 与 circuit_breaker/ 是两个模块吗？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [crate、self、super 与 use](paths-and-imports.md) · [pub 与受限可见性](visibility.md)
