# 静态与动态分发

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**泛型按具体类型生成调用，dyn 通过运行时接口调用；调用方式与谁拥有对象是两回事。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [Box：独占拥有堆上对象](../sharing/box.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：`select_with_policy / select_with_dynamic_policy` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

以下 &[Worker] 是阶段 3 的同步接口示意，方法体可能省略；当前源码使用 &[Arc<Worker>]。共享机制在阶段 7 再读，不要求把示意片段当作当前源码。

## 泛型：静态分发

统一调用入口可以写成泛型函数：

```rust
pub fn select_with_policy<P: Policy>(
    policy: &mut P,
    workers: &[Worker],
) -> Option<usize> {
    policy.select(workers)
}
```

`P: Policy` 是 trait bound，表示 `P` 必须实现 `Policy`。

它更接近 Java 的泛型工具方法：

```java
static <P extends Policy> Integer selectWithPolicy(
    P policy,
    List<Worker> workers
) {
    return policy.select(workers);
}
```

它不是工厂方法，因为它没有创建策略，只接收已有策略并调用其行为。

## 编译器如何处理泛型调用

调用两种具体类型时：

```rust
select_with_policy(&mut first_healthy, &workers);
select_with_policy(&mut round_robin, &workers);
```

可以近似理解为编译器分别生成两个具体版本：

```text
select_with_policy::<FirstHealthy>(...)
select_with_policy::<RoundRobin>(...)
```

这个过程通常称为单态化（monomorphization）。具体调用目标在编译期已知，因此叫静态分发。

## 静态分发的特点

| 特点 | 影响 |
|---|---|
| 编译期知道具体类型 | 容易内联和优化 |
| 不需要虚表查找 | 调用开销通常很小 |
| 每种具体类型可能生成一份代码 | 泛型实例较多时会增加编译时间和二进制体积 |
| 局部变量仍有唯一具体类型 | 同一个普通变量不能一会儿是 `FirstHealthy`、一会儿是 `RoundRobin` |

最后一点很关键：泛型表达“可接受多种类型”，不代表一个变量能在运行时随意改变具体类型。

## `impl Trait` 与静态分发

参数位置的 `impl Policy` 是泛型参数的简写之一：

```rust
fn select_with_policy(
    policy: &mut impl Policy,
    workers: &[Worker],
) -> Option<usize> {
    policy.select(workers)
}
```

它仍然是静态分发，不等于 `dyn Policy`。

返回位置的 `impl Policy` 表示函数隐藏一个确定的具体返回类型：

```rust
fn default_policy() -> impl Policy {
    FirstHealthy::new()
}
```

函数的所有返回路径仍必须解析为同一种具体类型。若要根据运行时配置返回不同具体策略，通常
改用 `Box<dyn Policy>`。

## `dyn Trait`：动态分发

动态入口可以接收 trait object：

```rust
pub fn select_with_dynamic_policy(
    policy: &mut dyn Policy,
    workers: &[Worker],
) -> Option<usize> {
    policy.select(workers)
}
```

这里调用方只知道“它实现了 `Policy`”，具体是 `FirstHealthy` 还是 `RoundRobin` 要到运行时从
trait object 的虚表中确定。

概念上，trait object 的指针包含两部分：

```text
&mut dyn Policy
├─ data pointer   -> 具体策略值
└─ vtable pointer -> 该具体类型的 Policy 方法表及相关元数据
```

调用 `policy.select(...)` 时，会通过虚表找到当前具体类型的实现。这就是动态分发。

这是概念模型；代码不应依赖 trait object 的具体内存布局细节。

## 为什么通常不能单独写 `dyn Policy`

不同实现的大小可能不同：

```text
FirstHealthy：没有字段
RoundRobin：包含 usize 计数器
其他策略：可能包含 Vec、HashMap 或配置
```

编译器无法为一个裸 `dyn Policy` 局部值确定固定栈大小。因此 `dyn Trait` 属于动态大小类型，
通常必须放在某种指针后面：

```rust
&dyn Policy
&mut dyn Policy
Box<dyn Policy>
Arc<dyn Policy>
```

这些指针自身大小固定，并携带访问具体值和虚表所需的信息。

## `&dyn Policy`：只借用，不拥有

如果 trait 方法只需要 `&self`，可以使用：

```rust
fn inspect(policy: &dyn Policy) {
    // 共享借用 trait object
}
```

当前 `Policy::select` 需要 `&mut self`，因此调用入口使用：

```rust
fn select(policy: &mut dyn Policy, workers: &[Worker]) {
    // 独占可变借用 trait object
}
```

借用形式的特点：

- 不取得具体策略的所有权；
- 一般不因为类型擦除而要求堆分配；
- 生命周期不能超过原策略；
- 调用结束后，原所有者继续拥有策略。

引用是借用，不是拥有型智能指针。这里把它列出，是因为 trait object 必须通过某种指针形式
使用，而引用是最轻量的选择。

## 静态分发与动态分发对照

| 维度 | `P: Policy` / `impl Policy` | `dyn Policy` |
|---|---|---|
| 确定实现 | 编译期 | 运行时 |
| 调用方式 | 直接调用具体实现 | 通过虚表间接调用 |
| 内联机会 | 通常更好 | 通常较少 |
| 代码体积 | 每个具体类型可能单态化 | 调用路径更集中 |
| 同一变量运行时换实现 | 不可以直接换 | 配合 Box 等可以 |
| 异构容器 | 不直接支持 | `Vec<Box<dyn Policy>>` |
| 是否必须堆分配 | 否 | `&dyn Trait` 不需要；`Box<dyn Trait>` 需要 |
| 常见使用场景 | 算法、内部组件、性能敏感路径 | 插件、配置驱动实现、稳定抽象边界 |

不要仅凭“动态分发有一次间接调用”就提前优化。网关中的网络、序列化和排队成本通常远高于
一次虚表调用；应根据所有权、扩展性和实测结果选择。

## 自查与关联

Box<dyn Policy> 中 Box 与 dyn 各负责什么？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
