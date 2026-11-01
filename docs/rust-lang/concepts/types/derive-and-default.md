# derive 与 Default：编译器替你生成什么

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**derive 自动生成指定 trait 的实现，不会自动选择业务上正确的语义。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`#[derive(Default)]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## Rust 内置

无需添加依赖：

```text
Debug、Clone、Copy、Default
PartialEq、Eq、PartialOrd、Ord、Hash
```

## 最常见组合

此表用于回查能力名字；Serialize、Deserialize 在阶段 4 介绍，thiserror、clap 为可选扩展，不作为阶段 3 的学习要求。

| 场景 | 常用组合 |
|---|---|
| 普通配置结构体 | `Debug, Clone, Default` |
| 值对象 / ID | `Debug, Clone, PartialEq, Eq, Hash` |
| 小型无资源状态枚举 | `Debug, Clone, Copy, PartialEq, Eq` |
| 可排序 key | `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` |
| JSON / 配置数据 | `Debug, Clone, Serialize, Deserialize` |
| 应用错误 | `Debug, thiserror::Error` |
| CLI 参数 | `Debug, clap::Parser` |

不要把这张表当成固定模板。只派生代码真正需要的能力，尤其要谨慎对待 `Copy`、`Default`、
`Ord` 和面向外部协议的 `Serialize` / `Deserialize`。

## `Default`：提供默认值

```rust
#[derive(Debug, Default)]
struct RetryConfig {
    max_retries: usize, // 0
    enabled: bool,      // false
    endpoint: String,   // ""
}

let config = RetryConfig::default();
```

结构体的派生实现逐字段调用 `Default::default()`。如果业务默认值不是各字段的零值，应手写：

```rust
impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            enabled: true,
            endpoint: "http://127.0.0.1:3000".to_owned(),
        }
    }
}
```

枚举需要用 `#[default]` 指定默认的 unit variant：

```rust
#[derive(Debug, Default)]
enum Mode {
    #[default]
    Auto,
    Manual,
}
```

能派生不代表语义合理。如果“不提供值”应该报错，就不应为了方便构造而添加 `Default`。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | RetryConfig 派生 Default 后各字段取什么值？ |
| 解释 | new 与 Default 为什么不必一致？ |
| 边界 | 业务必须提供 endpoint 时应为方便而添加空字符串默认值吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 阶段 4 后：第三方宏与诊断

以下声明是既有独立示例；thiserror、clap 没有引入当前项目，不要求照着添加依赖。

## 第三方过程宏

由依赖 crate 提供，通常需要开启相应 feature：

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
clap = { version = "4", features = ["derive"] }
```

```rust
use clap::Parser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Parser)]
struct Args { /* ... */ }

#[derive(Serialize, Deserialize)]
struct Config { /* ... */ }

#[derive(Debug, Error)]
enum AppError { /* ... */ }
```

相同的 `#[derive(...)]` 语法，背后可能是编译器内置能力，也可能是 crate 导出的过程宏。

## 泛型类型的 trait bound

自动派生会根据字段生成所需约束：

```rust
#[derive(Debug, Clone)]
struct Envelope<T> {
    value: T,
}
```

生成的实现近似为：

```rust
impl<T: Debug> Debug for Envelope<T> { /* ... */ }
impl<T: Clone> Clone for Envelope<T> { /* ... */ }
```

所以 `Envelope<NotClone>` 类型本身仍然可以存在，只是不能调用 `.clone()`。如果宏推导的约束
过强或不符合语义，可以手写 trait 实现；Serde 等第三方宏也通常提供属性来覆盖 bound。

## 某个字段没有实现目标 trait

```text
the trait bound `SomeField: Clone` is not satisfied
```

派生实现会递归要求相关字段实现该 trait。可选做法是：

- 给字段类型也实现或派生该 trait；
- 删除不需要的 derive；
- 改用 `Arc<T>` 等符合所有权语义的类型；
- 手写实现，只处理真正属于该 trait 语义的字段。

## 找不到第三方 derive 宏

```text
cannot find derive macro `Serialize` in this scope
```

依次检查：

1. `Cargo.toml` 是否添加依赖并开启 `derive` feature；
2. 是否 `use serde::Serialize;`；
3. 或者改用完整路径 `#[derive(serde::Serialize)]`。

## 派生前的检查清单

```text
Debug       输出是否可能泄露敏感数据？
Clone       复制成本是否可以接受？
Copy        隐式复制是否符合类型的资源语义？
Default     零值是否真的是合法业务默认值？
Eq / Hash   哪些字段定义对象身份？两者是否一致？
Ord         字段声明顺序是否就是业务排序顺序？
Serde       这是否会形成需要长期兼容的外部协议？
Error       source 和自动 From 转换是否准确？
```

## 自查与关联

derive(Default) 与手写 new 是否必然一致？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
