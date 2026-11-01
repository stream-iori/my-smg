# ref、引用模式与解引用

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**ref 在模式中借用匹配内容，& 在表达式中借用值，* 在表达式中访问解引用目标。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md) · [match 分支与守卫的执行顺序](match-and-guards.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`if let Some(handle) = &self.handle` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 在模式里借用值

独立语法例子，只比较模式绑定：

```rust
let slot = Some(Box::new(42));
if let Some(ref boxed) = slot {
    assert_eq!(**boxed, 42);
}
assert!(slot.is_some());
```

| 写法 | 作用 |
|---|---|
| `Some(boxed)` 匹配拥有型 slot | 默认按值绑定，Box 会被移出 |
| `Some(ref boxed)` 匹配 slot | boxed 是 &Box<i32>，不取走 Box |
| `Some(boxed)` 匹配 &slot | 借助匹配借用规则，boxed 也是借用 |

项目 Drop 中使用 `if let Some(handle) = &self.handle`，因此只借用句柄，避免从即将析构的对象移走字段。
Rust 2024 限制了某些混用隐式借用模式与显式 ref 的写法；本例在拥有型表达式上使用 ref，不要无条件在已有引用模式上再加 ref。

## 自查与关联

为什么 Drop 里只借用 handle 而不移走？

[match 分支与守卫的执行顺序](match-and-guards.md) · [if let 与 let else：成功值在哪里可用](if-let-and-let-else.md) · [解构：从 State、结构体和元组中取值](destructuring.md)
