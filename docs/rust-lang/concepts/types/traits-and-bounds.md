# Trait：类型需要提供什么能力

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**trait 约定能力，约束要求传来的类型实现它；trait 实现不等于 Java 类继承。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](generics-and-inference.md) · [共享借用与可变借用](../ownership/borrowing.md) · [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：`Policy / impl Policy for RoundRobin` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

以下 &[Worker] 是阶段 3 的同步接口示意，方法体可能省略；当前源码使用 &[Arc<Worker>]。共享机制在阶段 7 再读，不要求把示意片段当作当前源码。

## 从两个策略开始

假设模型网关有两种选择策略：

```rust
pub struct FirstHealthy;

pub struct RoundRobin {
    next: usize,
}
```

它们内部状态不同：

- `FirstHealthy` 没有字段，每次选择第一个健康节点；
- `RoundRobin` 保存 `next`，每次选择后修改计数器。

调用方真正关心的是共同能力：给定节点列表，返回一个可用节点的原始索引。

```rust
pub trait Policy {
    fn select(&mut self, workers: &[Worker]) -> Option<usize>;
}
```

这就是行为契约。它规定：

- 调用需要策略的可变借用 `&mut self`；
- 节点列表只被共享借用；
- 成功时返回原始节点索引；
- 没有可用节点时返回 `None`。

trait 不规定实现是否使用计数器、哈希、随机数或请求信息。

## Trait 与 Java interface 的类比

可以先把 Rust trait 类比为 Java interface：类型通过实现共同接口，让调用方依赖契约而不是
具体实现。

```rust
impl Policy for FirstHealthy {
    fn select(&mut self, workers: &[Worker]) -> Option<usize> {
        // FirstHealthy 的实现
    }
}

impl Policy for RoundRobin {
    fn select(&mut self, workers: &[Worker]) -> Option<usize> {
        // RoundRobin 的实现
    }
}
```

但两者不能完全等同：

| 方面 | Rust trait | Java interface |
|---|---|---|
| 常见分发方式 | 泛型静态分发或 `dyn` 动态分发 | 对象方法通常动态分发 |
| 实现位置 | 可为已有类型实现 trait，但受 orphan rule 限制 | 类声明 `implements` |
| 数据大小 | `dyn Trait` 本身是动态大小类型 | 对象变量本来就是引用 |
| 组合约束 | `T: A + B`、`dyn A + Send + Sync` | `T extends A & B` 等 |
| 关联能力 | 关联类型、关联常量、泛型方法 | 泛型与默认方法等机制不同 |

当前阶段最重要的差别是：Rust 只有显式写出 `dyn Policy` 时，才明确要求 trait object 的动态
分发。

表中提到的“关联类型”不是普通泛型参数。例如 Future 在 trait 中声明 `type Output;`，
具体实现确定完成结果的类型；`Future<Output = usize>` 则要求该结果为 usize。
完整语法与当前重试例子见 [类型别名与关联类型](associated-types.md)。

## 找不到 trait 提供的方法

如果类型实现了 trait，但调用处报方法不存在，检查 trait 是否已进入当前作用域：

```rust
use crate::policy::Policy;
```

## 自查与关联

为什么实现了 Policy 之后还可能需要把 trait 引入作用域？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [关联类型：实现确定自己的 Output](associated-types.md) · [类型别名与新类型](aliases-and-newtypes.md)
