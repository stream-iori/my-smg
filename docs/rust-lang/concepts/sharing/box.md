# Box：独占拥有堆上对象

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Box 是拥有型指针，可以容纳动态大小的 trait 对象；解引用不会自动复制或移动一切。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md) · [Drop 与 RAII：离开作用域如何收尾](../ownership/drop-and-raii.md) |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：`Box<dyn Policy + Send>` |
| 阅读时机 | 阶段 3 先读基础部分，再结合 trait 学习动态策略。 |


## 先只看 Box<T>

Box<T> 独占拥有堆上的 T。Box::new(42) 是已有材料中的独立整数例子，不依赖 trait 或网络库。Box 销毁时负责销毁内部值并释放存储；借用内部值不增加拥有者。Box 不提供共享计数或锁。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | Box 按值交给新变量后，旧变量还能用吗？ |
| 解释 | 独占拥有与临时借用有什么区别？ |
| 边界 | 只放进 Box 就能让多个线程随意修改内部值吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 项目应用：阶段 3 的动态策略；Send 在阶段 7 回查

另需[trait](../types/traits-and-bounds.md)。

## Box 先负责拥有，dyn 再负责接口调用

项目在 `src/main.rs` 中选择策略，类型声明为：

```rust
let policy: Box<dyn Policy + Send> = match &config.policy {
    PolicyKind::FirstHealthy => Box::new(FirstHealthy::new()),
    PolicyKind::RoundRobin => Box::new(RoundRobin::new()),
};
```

这是当前代码摘录，依赖所在模块的 imports。

```text
Box：独占拥有策略值，销毁时负责释放
 dyn Policy：调用者只依赖 Policy 接口
 Send：这个策略值能够被转移到其他线程
```

Box 本身不提供共享拥有，也不提供锁。它的解引用目标是内部 T；独立例子 `Box::new(42)` 可以被借用为 `&i32`，不会因为解引用就自动复制整个对象。

## `Box<dyn Policy>`：独占所有权与运行时切换

`Box<T>` 在堆上保存一个 `T`，并独占拥有它。`Box` 离开作用域时，内部值会被释放。

```rust
let mut policy: Box<dyn Policy> = Box::new(FirstHealthy::new());

policy = Box::new(RoundRobin::new());
```

同一个变量可以先拥有 `FirstHealthy`，再拥有 `RoundRobin`，因为变量的静态类型始终没有改变：

```text
变量类型始终是 Box<dyn Policy>
具体实现从 FirstHealthy 切换为 RoundRobin
```

重新赋值时，原来的 Box 及其内部策略会先被释放。

调用接收 `&mut dyn Policy` 的函数时，可以写：

```rust
select_with_dynamic_policy(policy.as_mut(), &workers)
```

类型变化为：

```text
Box<dyn Policy>
      │ .as_mut()
      ▼
&mut dyn Policy
```

`Box` 负责拥有策略，`&mut dyn Policy` 只在这次调用期间借用它。

## 何时适合 `Box<dyn Trait>`

- 根据配置在运行时选择一种实现；
- 容器需要保存多种实现，例如 `Vec<Box<dyn Middleware>>`；
- API 希望隐藏具体实现类型；
- 实现只有一个所有者，不需要跨多个组件共享所有权。

## 自查与关联

Box 拥有权与 dyn 的分发方式能否分开理解？

[Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md) · [RefCell：单线程里的运行时借用检查](refcell.md)
