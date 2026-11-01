# Deref：为什么能写 *guard

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**守卫的解引用让代码访问受保护的数据；* 得到位置，是否移动还取决于类型与上下文。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Mutex 与 Guard：锁何时释放](mutex-and-guards.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`*remaining -= 1` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 用当前假节点的计数器看 *guard

`src/bin/fake-worker.rs` 中，remaining 是加锁得到的 MutexGuard<usize>：

```rust
if *remaining == 0 {
    false
} else {
    *remaining -= 1;
    true
}
```

这是持锁代码块内的源码片段。

```text
remaining：MutexGuard<usize>
       │ Deref / DerefMut
       ↓
锁保护的 usize
```

`*remaining == 0` 读取整数，`*remaining -= 1` 写入整数；都没有把守卫移走。
Deref 可以理解为“这个包装允许访问哪个目标”；DerefMut 还允许修改目标。自动解引用让方法调用可以穿过包装，但泛型 trait 约束不总会自动做同样的转换。

| 类型 | 解引用目标 | 典型用途 |
|---|---|---|
| Box<T> | T | 独占拥有对象 |
| MutexGuard<T> | T | 访问锁内数据，守卫负责解锁 |
| String | str | 借用文本视图 |

对于非 Copy 的目标，不能因为能写 *guard 就随意移出它；守卫仍需要保持原位置有效。

## `MutexGuard` 为什么能用 `*` 解引用

`MutexGuard<T>` 是 RAII 锁守卫，同时具有智能指针式访问能力。它实现了：

```text
Deref<Target = T> 读取内部 T
DerefMut           修改内部 T
Drop               离开作用域时释放锁
```

因此：

```rust
remaining: MutexGuard<'_, usize>
*remaining: usize
```

可以画成：

```text
remaining
MutexGuard<usize>
        │ *
        v
锁保护的 usize
```

这与 `Box<T>` 的解引用形式相似：

```rust
let boxed = Box::new(10);
let value = *boxed;
```

区别是 `MutexGuard` 还代表锁的持有状态。

## 读取

```rust
if *remaining == 0 {
    // 比较内部 usize
}
```

不能直接写 `remaining == 0`，因为那是在比较 `MutexGuard<usize>` 和整数。

## 修改

```rust
*remaining -= 1;
```

可以近似理解为：

```rust
*remaining = *remaining - 1;
```

这需要两个条件：

```text
MutexGuard 实现 DerefMut
remaining 绑定声明为 mut
```

所以代码必须是：

```rust
let mut remaining = /* MutexGuard */;
*remaining -= 1;
```

`mut` 不是为了替换 Guard，而是为了通过 Guard 取得内部数据的可变访问。

## 解引用不一定意味着移动数据

`usize` 实现 `Copy`，因此读取会复制数值：

```rust
let count = *remaining;
```

如果 Mutex 内部是不可 `Copy` 的 `String`，不能直接把值从 Guard 后面移动出来：

```rust
// let text = *guard;       // 尝试移动 String，通常无法编译
let text_ref = &*guard;     // 借用 String
let text = (*guard).clone(); // 明确复制 String
```

所有权、`Copy` 和移动的区别见[所有权、转移与借用](../ownership/move-and-partial-move.md)。

## 方法调用中的自动解引用与自动借用

**自动解引用帮助找到目标类型的方法；自动借用根据接收者要求传入引用。两者不改变原变量声明的类型。**

以 `worker: Arc<Worker>` 为例：

| 调用 | 方法需要的接收者 | 怎样理解 |
|---|---|---|
| `worker.counter()` | `&Worker` | Arc 实现 `Deref<Target = Worker>`，调用可以访问目标类型的方法并借用目标 |
| `worker.begin_request_owned()` | `&Arc<Worker>` | 这个方法要求借用 Arc 句柄，调用时自动补上共享借用 |

```text
访问目标：Arc<Worker> --解引用访问目标 + 借用--> &Worker
访问句柄：Arc<Worker> --------借用句柄--------> &Arc<Worker>
```

不要把每次调用都理解成“先变成 Worker”。需要哪个接收者，取决于被选中的方法签名。
这里展示可用的接收者调整，不是完整的方法查找优先级规则。
展开调用语法和普通函数参数的区别见 [方法接收者](../ownership/method-receivers.md#点号调用会自动借用接收者)。

## 自查与关联

*guard 是取值、复制，还是写入位置？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
