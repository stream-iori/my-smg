# 类型推断、parse、collect 与空 Vec

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**上下文能确定类型时可以省略；turbofish 是显式补充类型参数，不是运行时传参。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [数组、切片、Vec 与 usize](../values/arrays-slices-and-vec.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`parse::<u64>` |
| 阅读时机 | T03 按需读到基础自查；解析和收集在阶段 4 后阅读。 |

## `Vec::<usize>::new()` 拆开看

| 部分 | 含义 |
|---|---|
| `Vec` | 标准库的动态数组类型 |
| `usize` | 这个 `Vec` 保存的元素类型 |
| `::new()` | 调用 `Vec` 的关联函数 `new` 创建空值 |
| `::<usize>` | 为泛型参数指定类型的 turbofish 语法 |

因此：

```rust
let indices = Vec::<usize>::new();
```

等价于把类型写在变量一侧：

```rust
let indices: Vec<usize> = Vec::new();
```

两者都会创建一个长度为 0、元素类型为 `usize` 的 `Vec`。

## 为什么空 `Vec` 经常需要类型提示

下面的值没有元素：

```rust
let values = Vec::new();
```

如果后续代码也没有提供信息，编译器无法判断它究竟是 `Vec<usize>`、`Vec<String>`，还是其他
类型的 `Vec`。这时需要显式标注类型。

有元素或周围上下文足够时，编译器通常能推断：

```rust
let indices = vec![0_usize, 2_usize];

let mut indices = Vec::new();
indices.push(0_usize); // 从 push 的参数推断为 Vec<usize>
```

函数返回类型也能提供上下文：

```rust
fn empty_indices() -> Vec<usize> {
    Vec::new()
}
```

这里不需要 turbofish，因为 `-> Vec<usize>` 已经告诉编译器返回值的具体类型。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 空 Vec 后续 push 一个 usize，能推断出什么元素类型？ |
| 解释 | 变量注解与函数返回类型如何帮助推断？ |
| 边界 | 空 Vec 没有后续类型信息，编译器能任选一种类型吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 阶段 4 后：泛型解析与收集

另需[泛型](generics-and-inference.md)、[Result](../errors/option-and-result.md)、[迭代](../values/iteration-and-collection.md)；使用 ? 前读[错误传播](../errors/question-mark.md)。pending 例子在阶段 5 阅读。

## turbofish：给泛型指定类型

本次用到的 turbofish 形状是 `::<类型>`：在表达式中明确指定泛型类型参数，
主要用于编译器推断不出来，或希望读代码时直接看到目标类型的场景。

| 场景 | 示例 | 指定什么 |
|---|---|---|
| 字符串解析 | `"123".parse::<u64>()` | 解析目标类型 `u64` |
| 创建空容器 | `Vec::<usize>::new()` | 容器元素类型 `usize` |
| 收集迭代器 | `items.collect::<Vec<_>>()` | 收集到 Vec，元素类型让编译器推断 |
| 调用泛型函数 | `std::future::pending::<()>()` | Future 的输出类型 `()` |

这里的 `items` 必须是迭代器；`Vec<_>` 中的 `_` 是类型推断占位符，并不是某种具体类型。
例如：

```rust
let values = [1, 2, 3].into_iter().collect::<Vec<_>>();
```

上下文足够明确时可以省略；下面没有 turbofish，但变量类型已经提供了解析目标：

```rust
let value: u64 = "123".parse().expect("valid number");
```

记住：尖括号中是编译期泛型参数，不是传给函数的运行时数据，也不是强制类型转换。
类型位置通常写 `Vec<usize>`；表达式位置指定参数时写 `Vec::<usize>::new()`。

## `parse::<u64>()`：给泛型方法指定目标类型

Turbofish 不只用于 `Vec` 这样的泛型类型，也能用于泛型方法：

```rust
let delay_ms = delay_ms_text.parse::<u64>();
```

拆开看：

| 部分 | 含义 |
|---|---|
| `delay_ms_text` | 等待解析的字符串 |
| `.parse()` | 泛型解析方法 |
| `::<u64>` | 指定解析目标类型为 `u64` |
| 返回值 | `Result<u64, ParseIntError>` |

数据流：

```text
"500": String
      │ parse::<u64>()
      v
Ok(500): Result<u64, ParseIntError>
```

`parse` 可以生成多种类型，简化后的函数形状可以理解为：

```rust
parse::<目标类型>() -> Result<目标类型, 解析错误>
```

所以同一个字符串可以选择不同目标：

```rust
"500".parse::<u64>()
"3.14".parse::<f64>()
"true".parse::<bool>()
```

如果上下文已经明确目标类型，可以省略 turbofish：

```rust
let delay_ms: u64 = delay_ms_text.parse()?;
```

两种常见位置对照：

| 写法 | 泛型参数指定给谁 |
|---|---|
| `Vec::<usize>::new()` | 泛型类型 `Vec` |
| `text.parse::<u64>()` | 泛型方法 `parse` |

核心不变：当编译器无法推断，或者希望代码直接表达目标类型时，用 `::<类型>` 明确告诉它。

## 阶段 4 后：其他推断上下文


Rust 会从上下文推断多种信息，但不同占位符含义不同：

| 写法 | 推断什么 |
|---|---|
| `Formatter<'_>` | 生命周期参数 |
| `Vec::<usize>::new()` | 显式指定泛型类型，不是推断 |
| `let values: Vec<usize> = Vec::new()` | 从变量类型推断 `Vec::new()` 的元素类型 |
| `serde_json::from_str(input)` | 从函数返回类型推断目标类型 |
| `Self::EmptyWorkers` | 从当前 `impl` 推断 `Self` 是哪个类型 |

例如：

```rust
fn from_json(input: &str) -> Result<GatewayConfig, serde_json::Error> {
    serde_json::from_str(input)
}
```

`from_str` 是泛型函数，但返回类型要求 `GatewayConfig`，所以编译器推断目标类型就是
`GatewayConfig`。

推断不是“运行时猜测”。所有结果都必须在编译期唯一确定，否则编译器会要求补充类型或生命
周期信息。
