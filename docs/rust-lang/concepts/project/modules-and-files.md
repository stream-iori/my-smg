# 模块声明如何找到文件

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**mod 建立模块树；目录存在不等于模块已经声明。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`mod retry` |
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

## 从 `mod retry;` 到 `gateway/retry.rs`

当前项目有两棵独立的编译树，库入口是 `lib.rs`，程序入口是 `main.rs`。

```text
程序 crate（main.rs）
└── gateway（gateway.rs）
    ├── chat（gateway/chat.rs）
    ├── chat_stream（gateway/chat_stream.rs）
    ├── gateway_state（gateway/gateway_state.rs）
    ├── management（gateway/management.rs）
    ├── retry（gateway/retry.rs）
    └── tests（gateway/tests.rs，仅测试构建）

库 crate（lib.rs）
├── circuit_breaker（circuit_breaker.rs）
└── circuit_gate（circuit_gate.rs）
```

| 声明位置 | 声明 | 默认查找文件 |
|---|---|---|
| `main.rs` | `mod gateway;` | `src/gateway.rs` |
| `gateway.rs` | `mod retry;` | `src/gateway/retry.rs` |
| `lib.rs` | `pub mod circuit_gate;` | `src/circuit_gate.rs` |

`gateway.rs` 与 `gateway/` 不是两个 gateway 模块。前者声明子模块，后者按默认约定存放子模块文件；目录本身不创建模块。
先有 `mod` 声明建立关系，再查找文件，不会自动扫描整个目录。

熔断状态机和共享入口现在是兄弟模块。共享入口使用：

```rust
use crate::circuit_breaker::{CircuitBreaker, CircuitState};
```

`crate` 指当前库的根；`super` 指当前模块在模块树中的父模块。它们都不表示运行时读取目录，也不创建另一份类型。
外部调用方通过 `my_smg::circuit_gate::CircuitGate` 访问共享入口。

## 把 mod、pub、use 分开看

| 写法 | 职责 |
|---|---|
| `mod retry;` | 声明模块并按约定查找定义 |
| `pub mod circuit_gate;` | 声明模块并公开模块路径 |
| `use crate::circuit_gate::CircuitGate;` | 引入已有类型的短名，不绕过可见性 |

程序中的网关处理器保留在二进制内部。测试放在网关的子模块中，因此无需为了测试把处理器暴露到公共库。
`tests` 这个名字没有特殊加载规则，`#[cfg(test)]` 才决定它仅在测试构建中加入。

## 与 mod.rs 写法的关系

| 模块 | 当前文件 | 另一种合法布局 |
|---|---|---|
| gateway | `src/gateway.rs` | `src/gateway/mod.rs` |
| gateway::retry | `src/gateway/retry.rs` | `src/gateway/retry/mod.rs` |
| circuit_gate | `src/circuit_gate.rs` | `src/circuit_gate/mod.rs` |

同一模块只能选择一种候选文件，两种布局不改变模块路径或访问权限。
