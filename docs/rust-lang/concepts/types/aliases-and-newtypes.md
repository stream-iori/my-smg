# 类型别名与新类型

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**type 只是另一个名字，struct 才建立新的类型边界。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [元组、元组结构体与 unit](../values/tuples-and-unit.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`HttpResponse` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## type HttpResponse：别名，不是新类型

当前 `src/gateway/retry.rs` 有：

```rust
type HttpResponse = (StatusCode, String);
```

```text
HttpResponse ──另一个名字──> (StatusCode, String)
```

它不创建变量、不分配内存、不包装元组，也不改变所有权。
这里的名字不是 Axum 的 Response，也不是 Reqwest 的 Response，只是我们给元组起的名字。

```rust
let response: HttpResponse = (StatusCode::OK, "hello".to_string());
let tuple: (StatusCode, String) = response;
```

不需要 `.into()`：二者本来就是同一个类型。上面赋值仍会移动含 String 的元组，
别名不会让它突然具备 Copy 能力。

| 写法 | 是否建立独立类型 | 是否有新包装层 |
|---|---|---|
| `type HttpResponse = (StatusCode, String);` | 否 | 否 |
| `struct HttpResponse(StatusCode, String);` | 是 | 有结构体边界 |

结构体需要通过构造表达式包装字段；别名没有独立构造器。
若要让编译器区分两种业务值，应该考虑新类型，而不是仅换别名。

## 自查与关联

为什么类型别名不需要 into 就能赋给原类型？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
