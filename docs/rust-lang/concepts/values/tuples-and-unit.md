# 元组、元组结构体与 unit

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**圆括号可以表示组合值、unit 或函数调用，要结合所在位置读。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | 无；可以直接阅读。 |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`type HttpResponse` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先看元素个数与位置

| 写法 | 含义 |
|---|---|
| () | 类型位置是 unit 类型，值位置是它唯一的值 |
| (42) | 整数加括号 |
| (42,) | 单元素元组，逗号不能省略 |
| (1, true) | 两个不同类型元素组成的二元组 |
| pair.0、pair.1 | 按位置访问二元组的元素 |

元组保存固定数量的值。unit 不携带额外信息，和函数永不返回不是一回事。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | ()、(42)、(42,) 分别是什么？ |
| 解释 | 为什么单元素元组要保留逗号？ |
| 边界 | 返回 unit 的函数能正常结束并继续调用方的后续执行吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 项目应用：阶段 5 后的响应与任务

另需[泛型](../types/generics-and-inference.md)、[Result](../errors/option-and-result.md)、[Future](../async/future-and-await.md)。

## 项目中的响应元组

```rust
type HttpResponse = (StatusCode, String);
```

这是项目的别名声明，`HttpResponse` 包含两个有顺序的值：

```text
HttpResponse
├── .0：StatusCode
└── .1：String
```

`(status, body)` 是元组值，`let (status, body) = response` 是解构模式。
与元组不同，`pub struct State<S>(pub S);` 是有名字的元组结构体：外层 pub 公开类型，内层 pub 公开第 0 个字段，S 是字段类型。

| 写法 | 位置与含义 |
|---|---|
| `()` | 类型位置：unit 类型，不携带额外信息 |
| `()` | 表达式位置：unit 的唯一值 |
| `(42,)` | 只有一个元素的元组；逗号不能省略 |
| `(42)` | 整数表达式加括号，不是元组 |
| `pending::<()>()` | 尖括号里的 () 是类型，末尾 () 是空参数调用 |
| `Ok(())` | Result 成功，成功值不携带额外数据 |

`src/gateway/tests/support.rs` 中，`TestServer` 保存 `ManagedTask<()>`：服务器任务完成时不产出业务值。unit 不是“这个函数永远不返回”；永不完成是 pending 的行为。
