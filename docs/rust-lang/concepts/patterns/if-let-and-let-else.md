# if let 与 let else：成功值在哪里可用

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**if let 的绑定在成功分支里，let else 的绑定在语句之后的当前块里。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../errors/option-and-result.md) · [match 分支与守卫的执行顺序](match-and-guards.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`let Some(index)` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `if let` 是什么

普通 `if` 判断 `bool`：

```rust
if is_ready { /* is_ready: bool */ }
```

`if let` 判断“值能否匹配模式”，并在匹配成功时声明模式中的变量：

```rust
let value = Some("hello");

if let Some(xx) = value {
    // xx: "hello"
}
```

```text
value = Some("hello")
       │
       ├─ 匹配 Some(xx) 成功 -> 声明 xx -> 执行 block
       └─ value 是 None      -> 跳过 block
```

它等价于只关心一个分支的 `match`：

```rust
match value {
    Some(xx) => { /* 使用 xx */ }
    _ => {}
}
```

所以 `Some(default_id)` 中：`Some` 是 `Option` 的变体，`default_id` 是在模式中声明的新变量。
`if let` 适合只处理一种情况；需要区分多个变体时使用 `match`。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | value 为 None 时会进入 if let 的成功分支吗？ |
| 解释 | 为什么成功分支绑定的名字不能在块外直接使用？ |
| 边界 | 能否把无值直接当成 Some(0) 而不检查？ |

先写下判断和原因，再用自己的已有代码验证。

## 阶段 2 按需：let-else；阶段 6 再看 HTTP 应用

let-else 也用于同步控制流，HTTP 状态码仅是后续示意，不是阶段 2 的实现要求。

## `let ... else`：取出值，失败就提前离开

`my-smg` 的节点选择返回 `Option<usize>`：`Some(index)` 表示选中了节点，`None` 表示没有
可用节点。以下省略 imports，将当前源码的 selected_idex 改写为 selected_index 解释形状：

```rust
let Some(index) = selected_index else {
    return (
        StatusCode::SERVICE_UNAVAILABLE,
        "no healthy worker".to_string(),
    );
};

// 从这里开始可以使用 index: usize
```

执行路径：

```text
selected_index: Option<usize>
        │
        ├─ Some(1) → 绑定 index = 1 → 继续执行
        └─ None    → 执行 else 中的 return → 结束整个 chat 函数
```

它相当于：

```rust
let index = match selected_index {
    Some(value) => value,
    None => {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "no healthy worker".to_string(),
        );
    }
};
```

这里的 `return` 返回的是**整个函数**的结果，不只是离开 `else`。`else` 分支必须明确离开
当前控制流，例如 `return`；不能在 `else` 中给 `index` 随意提供一个默认值后继续。

| 写法 | 匹配成功后，变量能用到哪里？ | 匹配失败时 |
|---|---|---|
| `if let Some(index) = value { ... }` | 只在 `{ ... }` 内 | 跳过该块，或进入 `else` 块 |
| `let Some(index) = value else { return ...; };` | 当前作用域后续代码 | 必须提前离开当前控制流 |

遇到 `Result<T, E>` 的失败传播时，还可以对照 [`?` 与提前返回](../errors/question-mark.md)；
`let ... else` 在这里用于把 `None` 映射成明确的 HTTP `503` 响应。

## 自查与关联

else 中为什么不能使用这次绑定的 index？

[match 分支与守卫的执行顺序](match-and-guards.md) · [解构：从 State、结构体和元组中取值](destructuring.md) · [ref、引用模式与解引用](reference-patterns.md)
