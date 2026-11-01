# PartialEq 与 Eq：怎样定义相等

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**PartialEq 允许部分等价关系，Eq 额外承诺自反性；派生比较会考虑字段。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](traits-and-bounds.md) · [derive 与 Default：编译器替你生成什么](derive-and-default.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker.rs](../../../../src/worker.rs)：`HealthStatus` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `PartialEq`：让类型可以使用 `==` 和 `!=`

没有实现 `PartialEq` 的自定义类型，不能直接使用 `==`：

```rust
// ❌ 这段代码无法编译
struct ModelKey {
    name: String,
    revision: String,
}

let left = ModelKey {
    name: "llama".to_owned(),
    revision: "v1".to_owned(),
};
let right = ModelKey {
    name: "llama".to_owned(),
    revision: "v1".to_owned(),
};

// 编译错误：ModelKey 没有实现 PartialEq
assert!(left == right);
```

派生 `PartialEq` 后就可以比较：

```rust
#[derive(Debug, PartialEq)]
struct ModelKey {
    name: String,
    revision: String,
}

let left = ModelKey {
    name: "llama".to_owned(),
    revision: "v1".to_owned(),
};
let same = ModelKey {
    name: "llama".to_owned(),
    revision: "v1".to_owned(),
};
let other_revision = ModelKey {
    name: "llama".to_owned(),
    revision: "v2".to_owned(),
};

assert!(left == same);
assert!(left != other_revision);
assert_eq!(left, same);
```

对于结构体，自动派生的规则是“所有对应字段都相等，两个值才相等”：

```text
left.name == right.name
    并且
left.revision == right.revision
    才能得到
left == right
```

近似于编译器生成：

```rust
impl PartialEq for ModelKey {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.revision == other.revision
    }
}
```

因此所有参与比较的字段也必须实现 `PartialEq`。`assert_eq!` 需要 `PartialEq` 来比较，并需要
`Debug` 在断言失败时打印左右值。

## enum 如何比较

```rust
#[derive(Debug, PartialEq)]
enum State {
    Waiting,
    Running { worker_id: u32 },
}

assert_eq!(State::Waiting, State::Waiting);
assert_eq!(
    State::Running { worker_id: 7 },
    State::Running { worker_id: 7 },
);
assert_ne!(
    State::Running { worker_id: 7 },
    State::Running { worker_id: 8 },
);
assert_ne!(State::Waiting, State::Running { worker_id: 7 });
```

两个 enum 值只有在 variant 相同、variant 内所有字段也相同时才相等。

## `Eq`：保证“每个值都等于自己”

`Eq` 容易让人困惑，是因为它不提供新的 `==` 方法。真正实现比较的是 `PartialEq`；`Eq` 是在
此基础上的一个保证：该类型的相等关系是完整的等价关系。

最直观的一条保证是：

```text
对任意值 x，都有 x == x
```

因此 `Eq` 依赖 `PartialEq`，一般一起派生：

```rust
#[derive(Debug, PartialEq, Eq)]
struct RequestId(u64);
```

可以这样记住两者的分工：

```text
PartialEq：实现“怎么比较”，提供 == 和 !=
Eq：额外承诺“这种比较没有例外，每个值都等于自己”
```

## 为什么浮点数只有 `PartialEq`，没有 `Eq`

浮点数有一个特殊值 `NaN`：

```rust
let value = f64::NAN;

assert!(value != value);
```

`NaN` 不等于它自己，违反了 `Eq` 的保证。所以：

```text
f64: PartialEq  ✓  可以写 1.0 == 1.0
f64: Eq         ✗  因为 NaN != NaN
```

包含 `f64` 字段的结构体可以派生 `PartialEq`，但不能派生 `Eq`：

```rust
#[derive(Debug, PartialEq)]
struct Score {
    value: f64,
}

// #[derive(Eq)] 会编译失败，因为 f64 没有实现 Eq。
```

这就是名字里 `Partial` 的核心含义：比较操作存在，但它不一定满足完整等价关系的全部规则。
这里的 `PartialEq::eq()` 仍然返回 `bool`，不是 `Option<bool>`。

## 为什么 `HashMap` 的 key 常要求 `Eq + Hash`

```rust
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
struct RequestId(u64);

let mut requests = HashMap::new();
requests.insert(RequestId(7), "running");

assert_eq!(requests.get(&RequestId(7)), Some(&"running"));
```

哈希表需要确信 key 的相等关系没有 `NaN` 这种例外，并要求相等的值产生相同哈希结果，所以 key
通常同时实现 `PartialEq + Eq + Hash`。

## 派生比较是否应该包含所有字段

自动派生会比较所有字段：

```rust
#[derive(Debug, PartialEq, Eq)]
struct Worker {
    id: u64,
    completed_requests: u64,
}
```

即使两个 worker 的 `id` 相同，只要统计值不同，它们也会被判定为不相等。如果业务身份只由
`id` 决定，可以拆出单独的 `WorkerId` 类型，或者谨慎地手写 `PartialEq`：

```rust
#[derive(Debug)]
struct Worker {
    id: u64,
    completed_requests: u64,
}

impl PartialEq for Worker {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Worker {}
```

如果该类型还实现 `Hash`，哈希也必须只使用相同的身份字段，否则会破坏哈希表规则。

## `PartialEq` 与 `Eq` 对照

| 类型 | `PartialEq` | `Eq` | 原因 |
|---|---:|---:|---|
| `u64`、`bool`、`String` | 是 | 是 | 每个值都等于自己 |
| `f32`、`f64` | 是 | 否 | `NaN != NaN` |
| 字段全是 `Eq` 的结构体 | 可派生 | 可派生 | 逐字段比较满足完整等价关系 |
| 含 `f64` 的结构体 | 可派生 | 不可派生 | 内部浮点值可能是 `NaN` |

## 自查与关联

两个 Arc 指向的值相等是否表示同一个对象？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
