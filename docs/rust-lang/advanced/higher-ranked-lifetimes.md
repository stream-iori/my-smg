# 高阶生命周期与函数输入逆变

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**固定一个生命周期与对任意生命周期成立是两种约束，不能机械混用。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [协变与不变：从 Java 泛型到 Rust 引用](variance.md) |
| 源码入口 | [src/gateway/retry.rs](../../../src/gateway/retry.rs)：`Future 约束的阅读前置` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 从固定范围走向所有范围

这篇是扩展，不是理解普通 async/await 的前置。项目重试目前不要求手写高阶生命周期约束。

| 写法 | 含义 |
|---|---|
| `fn(&'a str)` | 对上下文中选定的 a，参数有效 |
| `for<'a> fn(&'a str)` | 对任意调用所需的 a，都能接受相应引用 |
| `fn(&str)` | 函数指针中省略的引用生命周期涉及高阶绑定，不能机械当成某个固定 a |

固定生命周期上的逆变，与 for 所表达的普遍性不是同一步推理。先明确量词，再判断类型能否替换。
参考 [Rust Reference：高阶 trait bound](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)。

## Rust 的逆变：看函数参数

仍假设 `'long: 'short`，比较两个已经确定生命周期的函数指针类型：

```text
&'long str → &'short str

fn(&'short str) → fn(&'long str)
```

一个能处理短期引用的函数，也能处理长期引用；一个只能处理长期引用的函数，不能随便接收短期引用。

```rust
fn adapt<'long: 'short, 'short>(
    handler: fn(&'short str),
) -> fn(&'long str) {
    handler
}
```

输入越宽松，函数越能替代输入要求更严格的函数，所以输入位置是逆变。
返回值则相反：需要短期引用时，返回长期引用的函数可以胜任，所以输出位置协变。

这里有意写出生命周期。裸写 `fn(&str)` 涉及更高阶生命周期，不能机械当作某个固定的 `fn(&'short str)`。

## 自查与关联

fn(&str) 与某个固定 fn(&'a str) 能直接画等号吗？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md) · [扩展：thiserror 的错误派生](thiserror.md)
