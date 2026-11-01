# ? 与 ??：成功继续，失败提前返回

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Result 的 ? 传播 Err，Option 的 ? 传播 None；?? 逐层处理，不是新的运算符。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](option-and-result.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`operation().await?` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 写法二：返回值本来就是 `Option` 时使用 `?`

如果当前函数返回 `Option<Arc<dyn RouterTrait>>`，还可以写成：

```rust
let default_router = self
    .default_router
    .read()
    .unwrap_or_else(|e| e.into_inner());

let default_id = default_router.as_ref()?;
self.routers.get(default_id).map(|r| r.clone())
```

当默认路由为 `None` 时，`?` 会直接返回 `None`。

## `?` 无法转换错误

检查当前函数的错误类型与下层错误类型之间是否存在 `From` 实现。`Error` trait 本身不负责
转换。

## `?` 近似等价于什么

上面的代码可以展开为：

```rust
fn load_config(input: &str) -> Result<GatewayConfig, serde_json::Error> {
    let config = match GatewayConfig::from_json(input) {
        Ok(config) => config,
        Err(error) => return Err(error),
    };

    Ok(config)
}
```

重点是 `return Err(error)`：

- `return` 结束的是当前的 `load_config` 函数；
- 它不会自动结束整个程序；
- `load_config` 的调用者会收到这个 `Err`；
- 调用者可以处理它，也可以再次使用 `?` 继续向上传播。

真实的 `?` 还支持通过 `From` / `Into` 转换错误，因此“等价于 match”是帮助理解控制流的
近似模型，不是其全部实现细节。

## 为什么当前函数也必须允许返回错误

下面的函数声明返回普通 `GatewayConfig`：

```rust
fn load_config(input: &str) -> GatewayConfig
```

它无法直接使用一个可能产生 `serde_json::Error` 的 `?`，因为函数签名没有错误返回通道。

通常应把签名改成 `Result`：

```rust
fn load_config(input: &str) -> Result<GatewayConfig, serde_json::Error>
```

这样 `Err` 才能从当前函数返回给调用者。

看到类似编译错误时：

```text
the `?` operator can only be used in a function that returns `Result` or `Option`
```

应检查当前函数的返回类型，而不是简单地删除 `?` 或改成 `unwrap()`。

## 为什么成功值需要重新包进 `Ok`

使用 `?` 后，成功分支的 `Result` 外壳被打开：

```rust
let config = GatewayConfig::from_json(input)?;
// config: GatewayConfig，不是 Result<GatewayConfig, _>
```

如果当前函数声明返回 `Result<GatewayConfig, _>`，最终需要重新包装：

```rust
Ok(config)
```

类型变化是：

```text
Result<GatewayConfig, Error>
            │ ?
            ▼
GatewayConfig
            │ Ok(...)
            ▼
Result<GatewayConfig, Error>
```

之所以先取出再包装，通常是因为中间还需要校验、转换或执行其他可能失败的操作。如果什么都
不做，直接返回原来的 `Result` 更简单。

## 直接返回

```rust
fn from_json(input: &str) -> Result<Self, serde_json::Error> {
    serde_json::from_str(input)
}
```

适合内部调用的返回类型与当前函数完全相同，并且没有后续处理。

## 使用 `?`

```rust
fn from_json(input: &str) -> Result<Self, serde_json::Error> {
    let config = serde_json::from_str(input)?;
    // 在这里继续处理 config
    Ok(config)
}
```

适合成功后还需要继续执行校验、转换或其他步骤。

不要为了使用 `?` 而机械地拆开再包装；它的价值是简化多步骤失败控制流。

## `?` 也能用于 `Option`

**`?` 不只处理 Result，也能处理 Option。核心都是：成功就取值继续，失败就提前返回。**

| 表达式类型 | 继续执行 | 提前返回 | 常见的当前函数返回类型 |
|---|---|---|---|
| `Result<T, E>` | `Ok(value)` → 得到 value | `Err(error)` → 返回错误，必要时转换错误类型 | `Result<U, F>`，要求错误能转换为 F |
| `Option<T>` | `Some(value)` → 得到 value | `None` → 返回 None | `Option<U>` |

取出的 T 不必与当前函数最终返回的 U 相同，中间还可以继续处理。这里集中讨论 Result 和 Option；
这不是 `?` 支持类型的完整清单。见 [Rust Reference：try propagation expression](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression)。

当函数返回 `Option<T>` 时：

```text
Some(value) -> 取出 value，继续执行
None        -> 当前函数立即返回 None
```

例如：

```rust
fn first_name(names: &[String]) -> Option<&str> {
    let first = names.first()?;
    Some(first.as_str())
}
```

`Result` 的 `?` 传播 `Err`，`Option` 的 `?` 传播 `None`。相关模式示例见
[`if let`、`ref` 与解引用](question-mark.md)。

上述例子中，`names.first()` 返回 `Option<&String>`：

| 输入 | `names.first()` | `?` 后发生什么 | 函数结果 |
|---|---|---|---|
| 空切片 | `None` | 立即返回，不再执行下一行 | `None` |
| 至少有一个名字 | `Some(&String)` | 取出引用，继续调用 `as_str()` | `Some(&str)` |

## Option 与 Result 不能直接混用 `?`

**`?` 不会自动决定“缺失值应该算什么错误”，也不会自动丢弃错误信息。**

| 需求 | 先转换，再使用 `?` | 信息变化 |
|---|---|---|
| 在返回 Result 的函数中处理 Option | `option.ok_or(error)?` | 为 None 指定错误；Some 的值保持不变 |
| 错误需要按需构造 | `option.ok_or_else(\|\| error)?` | 只有 None 时才调用闭包构造错误 |
| 在返回 Option 的函数中处理 Result | `result.ok()?` | 丢弃 Err 中的错误信息，把它变成 None |

下面是两个独立语法例子：

```rust
fn first_number(values: &[i32]) -> Result<i32, &'static str> {
    let first = values.first().ok_or("empty input")?;
    Ok(*first)
}

fn parse_number(text: &str) -> Option<u32> {
    let number = text.parse::<u32>().ok()?;
    Some(number)
}
```

```text
Option → Result：None       → Err("empty input") → ? 提前返回错误
Result → Option：Err(error) → None               → ? 提前返回 None
                              ↑
                         错误原因被丢弃
```

只有确实不需要错误原因时才选 `.ok()`；若还需要区分失败原因，应保留 Result。
参见 [Option::ok_or](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or)、
[Option::ok_or_else](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else) 与
[Result::ok](https://doc.rust-lang.org/std/result/enum.Result.html#method.ok)。

## `?` 会处理错误

不准确。`?` 通常只是传播错误。日志记录、重试、转换为 HTTP 状态码等处理仍需由合适的层
完成。

## `?` 遇到错误会退出整个程序

不一定。它只让当前函数提前返回。是否最终结束程序，取决于上层如何处理返回的错误。

## 使用 `?` 后返回类型变成成功类型

局部变量会得到成功类型，但当前函数仍按照声明返回 `Result` 或 `Option`。

## 所有错误都能自动用 `?` 转换

不正确。源错误必须与目标错误相同，或者存在合适的 `From` 转换。

## `?` 可以放在任意函数中

不正确。当前函数或闭包的返回类型必须支持相应的提前返回语义。

## 自查与关联

Result 与 Option 为什么不能直接混用问号？

[Option 与 Result：有无与成败](option-and-result.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md) · [TryFrom、TryInto 与可能失败的转换](fallible-conversions.md)
