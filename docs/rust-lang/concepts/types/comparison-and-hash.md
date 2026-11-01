# 相等、排序与哈希

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**比较和哈希是类型的行为契约，不能只为让编译通过就随意派生。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [PartialEq 与 Eq：怎样定义相等](equality.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`#[derive(Debug, Deserialize, PartialEq, Eq)]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `PartialOrd` 与 `Ord`：排序

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Priority(u32);
```

结构体按字段声明顺序做字典序比较；枚举先按 variant 的声明顺序比较，再比较 variant 内字段。

```text
PartialOrd -> 可能无法比较，partial_cmp 返回 Option<Ordering>
Ord        -> 任意两个值都有确定顺序，cmp 返回 Ordering
```

因为浮点数存在 `NaN`，所以它们只有 `PartialOrd`，没有 `Ord`。

派生顺序未必等于业务顺序。例如版本号字符串按字典序比较时，`"10" < "2"`；调度优先级也可能
需要反向或多字段规则。这些情况应手写比较或使用合适的领域类型。

## `Hash`：作为哈希集合的 key

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct WorkerKey {
    model: String,
    address: String,
}

let mut workers = HashSet::new();
workers.insert(WorkerKey {
    model: "demo".to_owned(),
    address: "127.0.0.1:3000".to_owned(),
});
```

`HashMap` / `HashSet` 的 key 通常需要 `Eq + Hash`。两者必须保持一致：

```text
a == b  =>  hash(a) == hash(b)
```

同时派生 `PartialEq, Eq, Hash` 时，编译器会按相同字段生成实现，最不容易破坏这条约束。若手写
其中一个，必须确认另一个使用完全一致的身份字段。

## `Eq`、`Ord` 被浮点字段阻止

这通常不是“少写了一个 derive”，而是浮点数本身不具备完全相等或全序语义。应先定义业务上
如何处理 `NaN`，再选择包装类型或手写比较逻辑。

## 自查与关联

为什么浮点数会阻止某些 Eq 和 Ord 派生？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
