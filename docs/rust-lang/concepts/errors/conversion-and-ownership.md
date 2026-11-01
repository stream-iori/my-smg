# 转换与所有权：into 到底消费什么

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**into 按值接收接收者；接收者是引用时拿到的是引用值，不会因此取得原对象所有权。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：`error.into()` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## into 接收拥有值与引用的区别

`.into()` 则是 Into trait 的方法，按值接收接收者，并返回目标类型。接收者若是引用，
它拿到的只是引用值，不会因此拿走被引用对象的所有权。目标类型推断与 From / Into 的关系见
[类型转换专题](from-and-into.md)。

下面两组是独立的语法例子，目标类型都用 `String`，只比较传入拥有值和传入引用的区别。

**例一：接收者是 `String`，原值的所有权转走。**

```rust
let original = String::from("hello");
let result: String = original.into();

// original 已被移动，不能再使用
println!("{result}");
```

同类型之间也支持 `.into()`。这里没有复制字符串内容，只是把拥有值从 `original` 移到 `result`。

```text
转换前：original ──拥有──> "hello"

转换后：result   ──拥有──> "hello"
        original 失效
```

**例二：接收者是 `&str`，原字符串仍由 `original` 拥有。**

```rust
let original = String::from("hello");
let borrowed: &str = original.as_str();

let result: String = borrowed.into();

println!("{original}"); // 可以，原字符串没有被移动
println!("{borrowed}"); // 可以，共享引用实现了 Copy
println!("{result}");   // 可以，新字符串有自己的内容
```

这次 `.into()` 按值接收的是 `&str`。从 `&str` 转成 `String` 会复制字符串内容，得到独立的拥有值；它不会拿走 `original` 的所有权。

```text
original ──拥有──> "hello"（原内容）
                     ↑
borrowed ──借用───────┘

result   ──拥有──> "hello"（复制出的新内容）
```

| 对比项 | 例一：`original.into()` | 例二：`borrowed.into()` |
|---|---|---|
| 接收者类型 | `String` | `&str` |
| 目标类型 | `String` | `String` |
| 原字符串所有权是否转走 | 是 | 否 |
| 是否复制字符串内容 | 否 | 是 |
| `original` 能否继续使用 | 不能 | 能 |

**按值接收不代表原变量一定失效。** `String` 没有实现 `Copy`，例一发生移动；共享引用 `&str` 实现了 `Copy`，例二传入的是引用的副本，所以 `borrowed` 也能继续使用。复制引用不等于复制字符串；例二的字符串内容复制发生在 `&str` 转成 `String` 的过程中。

这些行为取决于具体源类型与目标类型，不能把 `.into()` 一概理解为“不复制”或“一定复制”。参见 [Into](https://doc.rust-lang.org/std/convert/trait.Into.html) 与 [String 的 From<&str> 实现](https://doc.rust-lang.org/std/string/struct.String.html#impl-From%3C%26str%3E-for-String)。

## 所有权与可逆性

`Into::into(self)` 消费接收者。传入非 Copy 的拥有型值时，它通常会移动，原变量不能继续使用。
传入 `&str` 时，消费的是可复制的引用，不会取得或销毁被借用文本的所有权。

转换不保证可逆。例如 `String::as_str()` 只是借用 String 保存的文本，不会撤销之前的
`&str` → `String` 转换。值转换、借用和克隆的关系见
[所有权文章中的方法接收者](../ownership/move-and-partial-move.md)。

## 自查与关联

为什么 String.into 和 str 引用.into 对原值的影响不同？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
