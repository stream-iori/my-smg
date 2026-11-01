# 哪些 trait 能作为 dyn Trait

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**动态接口必须能在不知道具体实现类型时调用，带某些泛型或 Self 用法的方法需要限制。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [静态与动态分发](static-and-dynamic-dispatch.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：`trait Policy` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

以下 &[Worker] 是阶段 3 的同步接口示意，方法体可能省略；当前源码使用 &[Arc<Worker>]。共享机制在阶段 7 再读，不要求把示意片段当作当前源码。

## Trait object 必须满足 dyn compatibility

不是所有 trait 都能直接写成 `dyn Trait`。编译器现在通常称这组规则为 dyn compatibility，
旧资料常称 object safety。

常见限制包括：

- trait 不能整体要求 `Self: Sized`；
- 要通过 trait object 调用的方法不能有自己的泛型类型参数；
- 方法不能在没有适当限制时按值返回未知的 `Self`；
- 方法需要具有可用于动态调用的接收者，例如 `&self`、`&mut self` 或 `Box<Self>`。

当前接口：

```rust
trait Policy {
    fn select(&mut self, workers: &[Worker]) -> Option<usize>;
}
```

可以构造 `dyn Policy`，因为它的方法满足动态调用所需条件。

如果编译器报告：

```text
the trait `...` is not dyn compatible
```

应先检查方法是否包含泛型参数、是否返回 `Self`，以及 trait 是否要求 `Sized`，而不是先尝试
加 Box 或 Arc。

## `the size for values of type dyn Trait cannot be known`

原因通常是尝试按值保存裸 `dyn Trait`：

```rust
// 错误方向：裸 dyn Trait 没有编译期固定大小
// let policy: dyn Policy = ...;
```

改为根据所有权需求选择 `&dyn Policy`、`Box<dyn Policy>` 或 `Arc<dyn Policy>`。

## 自查与关联

为什么不是任意 trait 都能写成 dyn Trait？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
