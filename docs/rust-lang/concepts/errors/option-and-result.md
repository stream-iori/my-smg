# Option 与 Result：有无与成败

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Option 表示有无；Result 表示成败并保留失败原因。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：Option<usize>；[src/config.rs](../../../../src/config.rs)：配置加载的 Result |
| 阅读时机 | T04 先读 Option；Result 在阶段 4 使用。 |

## Option：有值，或者没有值

| 值或类型 | 含义 |
|---|---|
| `Option<usize>` | 要么携带一个索引，要么没有索引 |
| `Some(0)` | 有索引，索引值为 0 |
| `None` | 没有索引，本身不携带错误原因 |

Some 与 None 是同一个枚举的变体。0 是合法索引，不能代替“未选中”。
Option 是普通值，不自动抛异常，也不自动变成内部整数。

```text
Option<usize>
    ├─ Some(index) → 有值分支
    └─ None        → 无值分支
```

两种情况都需要明确行为。[match](../patterns/match-and-guards.md)讲分支语法，
[if let/let-else](../patterns/if-let-and-let-else.md)讲只关心一种情况时如何写；它们是后续阅读，不是本篇前置。

## `Result<T, E>` 是什么

可以把标准库中的 `Result` 简化理解为：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

其中：

- `T` 是成功值的类型；
- `E` 是错误值的类型；
- `Ok(value)` 携带成功结果；
- `Err(error)` 携带失败原因。

```text
Result<T, E>
    ├─ Ok(value)  → 类型 T 的成功值
    └─ Err(error) → 类型 E 的错误值
```

与 Java 异常不同，Result 是签名中可见的普通返回值；它不会自动修复错误。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | Some(0)、None、Err(error) 分别表达什么？ |
| 解释 | 为什么没有选中索引与配置解析失败不宜都用 None？ |
| 边界 | 空列表能用 0 代表未选中吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

## 项目应用：阶段 4 后再读

Policy::select 返回 Option<usize>；配置加载通过 Result 保留失败原因。
传播错误见[问号](question-mark.md)。需要从可选字段借用或取走值时，读[容器借用与取值](option-access-and-take.md)，不用在初学 Option 时提前学习 Box。
