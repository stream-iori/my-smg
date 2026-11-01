# Copy 与 Clone：复制值还是共享句柄

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Copy 是隐式复制能力，Clone 是显式请求；clone 是否复制底层内容取决于类型。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Move 与部分移动](move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/circuit_breaker.rs](../../../../src/circuit_breaker.rs)：`CircuitState` |
| 阅读时机 | T06 读到基础自查；派生在阶段 3、Arc 在阶段 7 回查；Option 的 clone/cloned 和响应头应用在阶段 11 按需阅读。 |

## `Copy`：使用旧值时自动复制

先看整数：

```rust
let first: u32 = 10;
let second = first;

println!("{first}");  // 可以
println!("{second}"); // 可以
```

这段代码与 `String` 的例子看起来相同，但 `u32` 实现了 `Copy`。所以 `let second = first`
不会让 `first` 失效，而是自动复制一个值：

```text
赋值前：first = 10
赋值后：first = 10，second = 10
```

`Copy` 没有一个需要主动调用的 `.copy()` 方法。它是一个标记，告诉编译器：这个类型可以在
原本会 move 的地方自动复制。

## `Clone`：明确说“请再复制一份”

`Clone` 提供 `.clone()` 方法。调用者必须显式写出来：

```rust
let first = String::from("hello");
let second = first.clone();

println!("{first}");  // 可以
println!("{second}"); // 也可以
```

对于 `String`，`.clone()` 会申请新的堆内存并复制字符：

```text
first  ──拥有──> String("hello")，堆内存 A
second ──拥有──> String("hello")，堆内存 B
```

两个值互相独立：

```rust
let first = String::from("hello");
let mut second = first.clone();
second.push_str(" rust");

assert_eq!(first, "hello");
assert_eq!(second, "hello rust");
```

## 传参时也会触发 move 或 Copy

```rust
fn print_number(value: u32) {
    println!("{value}");
}

let number = 10;
print_number(number); // number 被 Copy 进函数
print_number(number); // 仍然可以使用
```

对没有实现 `Copy` 的 `String`，第一次按值传参后原变量就不能再使用，除非传引用或显式
`.clone()`：

```rust
fn print_text(value: &str) {
    println!("{value}");
}

let text = String::from("hello");
print_text(&text); // 借用，没有 move，也没有复制字符串
print_text(&text); // 可以继续借用
```

## 阶段 12：请求 ID 的 Copy 如何逐层成立

当前 RequestId 在 [src/main.rs](../../../../src/main.rs) 中通过 `#[derive(Clone, Copy, Debug)]` 显式声明复制能力，内部字段是支持 Copy 的 u64。Axum 的 Extension 泛型包装按 T 的能力有条件实现 Copy；Option 也一样。

```text
u64: Copy
    ↓ RequestId 显式 derive(Copy)，且字段满足条件
RequestId: Copy
    ↓ Extension<T> 在 T: Copy 时支持 Copy
Extension<RequestId>: Copy
    ↓ Option<T> 在 T: Copy 时支持 Copy
Option<Extension<RequestId>>: Copy
```

这不是编译器分析“以后是否需要独立生命周期”后自动决定的，而是类型作者选择公开 Copy 能力，并由编译器检查条件。仅仅所有字段支持 Copy，不会让自定义结构体自动实现 Copy。

| 考虑 | 请求 ID 的情况 |
|---|---|
| 语义是否允许多个相同的值 | 允许；数字副本表示同一个逻辑请求，不产生新请求 |
| 复制是否简单 | 是；复制一个数字，不分配堆内存 |
| 是否拥有必须唯一释放的资源 | 否；不同于文件句柄、并发凭证和资源 guard |
| 是否共享可变状态 | 否；修改一份数字不会修改另一份 |

Copy 是复制语义的承诺，不是生命周期标记，也不意味着比 move 更快。move 本身通常也很便宜；大型数组即使能 Copy，复制成本也会随数据量增长。

Extension<String> 不支持 Copy；Extension<Arc<T>> 通常支持 Clone，克隆的是共享句柄。实现 Drop 的类型不能实现 Copy，因此负责资源释放的 guard 不应被隐式复制。参见 [Axum 0.8.9 Extension 定义](https://docs.rs/axum/0.8.9/src/axum/extension.rs.html#71-72)。

关联：[Option::map 与按值调用](../errors/option-access-and-take.md#map-消费-self为什么原值有时还能用)。

## `Clone` 与 `Copy` 对照

| 问题 | `Clone` | `Copy` |
|---|---|---|
| 如何触发 | 显式调用 `.clone()` | 赋值、传参等位置隐式发生 |
| 是否可能昂贵 | 可能 | 应当是简单、便宜的值复制 |
| 能否用于拥有资源的类型 | 可以，例如 `String`、`Vec<T>` | 通常不可以 |
| 原变量能否继续使用 | 调用 `.clone()` 后可以 | 自动可以 |
| 能力关系 | 可以单独实现 | 必须同时实现 `Clone` |

判断过程：

```text
只想借来使用，不需要新值？ -> 传 &T / &mut T
需要明确产生另一个值？     -> Clone
类型是简单小值，隐式复制符合直觉？ -> Clone + Copy
```

`Copy` 的硬性约束包括：所有字段都是 `Copy`、类型同时实现 `Clone`，并且类型没有实现
`Drop`。常见的 `Copy` 类型有整数、浮点数、`bool`、`char`、共享引用 `&T`，以及只包含这些
字段的小型结构体或枚举。

含 String 字段的结构体不能派生 Copy，因为字段不全是 Copy；后文再解释派生。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | u32 赋值后原变量还能用吗？String 不 clone 时呢？ |
| 解释 | 为什么 String::clone 得到独立内容，而普通赋值不复制文本？ |
| 边界 | 一个含 String 的结构体能靠派生 Copy 允许隐式复制吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

## 阶段 3：派生自定义类型能力

先读[trait](../types/traits-and-bounds.md)、[derive](../types/derive-and-default.md)。

## `#[derive(Clone)]` 做了什么

```rust
#[derive(Clone, Debug)]
struct Config {
    endpoint: String,
    retries: u32,
}
```

派生实现近似于：

```rust
impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            retries: self.retries.clone(),
        }
    }
}
```

也就是逐字段调用 `.clone()`。只要有一个字段没有实现 `Clone`，整个结构体就不能自动派生
`Clone`。

## 给自定义类型派生 `Copy`

```rust
#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

let first = Point { x: 10, y: 20 };
let second = first;

println!("{first:?}");  // 可以，发生了 Copy
println!("{second:?}");
```

`i32` 是 `Copy`，所以只包含 `i32` 的 `Point` 也可以派生 `Copy`。

下面这个类型则不能派生 `Copy`：

```rust
// ❌ 这段代码无法编译
#[derive(Clone, Copy)]
struct User {
    name: String, // String 不是 Copy
}
```

如果允许按位复制 `String`，两个值就会保存相同的堆指针，并在离开作用域时尝试释放同一块内存。
因此，拥有堆内存、文件句柄或引用计数等资源的类型通常不能实现 `Copy`。

## 阶段 7：共享句柄的 Clone

先读[Arc](../sharing/rc-and-arc.md)。

## `clone()` 不一定是深拷贝

复制的具体含义由字段类型决定：

| 类型 | `.clone()` 通常做什么 |
|---|---|
| `String` | 申请新内存，复制字符串内容 |
| `Vec<T>` | 申请新内存，逐个 clone 元素 |
| `u32`、`bool` | 直接复制值 |
| `Arc<T>` | 只增加强引用计数，两个 `Arc` 仍指向同一个 `T` |

```rust
use std::sync::Arc;

let first = Arc::new(String::from("hello"));
let second = first.clone();

assert!(Arc::ptr_eq(&first, &second)); // 指向同一个 String
assert_eq!(Arc::strong_count(&first), 2);
```

所以 `Clone` 只表示“可以显式得到另一个值”，不保证复制便宜，也不保证底层数据完全独立。
示例中的 `Arc::ptr_eq` 检查对象身份，不是内容相等，详见
[Arc：对象身份与值相等](../../walkthroughs/circuit-permit.md)。


综合题：Arc::clone 与 String::clone 都会复制底层文本吗？

## Option 的 clone 与 cloned

**`clone()` 克隆当前接收者的值；对 `Option<&T>` 调用 `cloned()`，会克隆里面引用指向的 `T`。**

这里讨论的是 `Option` 上的调用。先读 [Option 与 Result](../errors/option-and-result.md)，必要时回查 [Option 中的借用](../errors/option-access-and-take.md)。

| 接收者 | 操作 | 得到的类型 | 含义 |
|---|---|---|---|
| `String` | `.clone()` | `String` | 得到拥有自己文本的字符串 |
| `Option<String>` | `.clone()` | `Option<String>` | `Some` 时克隆内部 String；`None` 时仍为 None |
| `Option<&String>` | `.clone()` | `Option<&String>` | `Some` 时复制内部引用，仍然借用原字符串 |
| `Option<&String>` | `.cloned()` | `Option<String>` | `Some` 时克隆引用指向的 String，得到拥有的值 |

### 示例一：复制引用与复制字符串

下面是可独立运行的教学示例，不是项目实现代码。类型标注用来明确两种结果：

```rust
fn main() {
    let original = String::from("hello");
    let borrowed: Option<&String> = Some(&original);

    let same_reference: Option<&String> = borrowed.clone();
    let mut owned: Option<String> = borrowed.cloned();

    // clone 后的引用仍指向原来的 String。
    assert!(std::ptr::eq(same_reference.unwrap(), &original));

    // cloned 得到自己的 String，可以独立修改。
    owned.as_mut().unwrap().push_str(" rust");

    assert_eq!(original, "hello");
    assert_eq!(owned.as_deref(), Some("hello rust"));

    println!("原字符串：{original}");
    println!("克隆得到的字符串：{owned:?}");
}
```

输出：

```text
原字符串：hello
克隆得到的字符串：Some("hello rust")
```

示例中的 `unwrap()` 用在已知为 `Some` 的教学值上，方便观察类型；业务中的可选值仍需处理 `None`。
`Option<&String>` 本身也实现了 `Copy`，实际代码可直接赋值来复制它；这里显式写 `.clone()` 是为了比较两个方法。

```text
borrowed       --引用--+
                      +--> original 拥有的 String("hello")
same_reference --引用--+

owned --拥有--> 另一份 String("hello rust")
```

`cloned()` 对 `Option<&T>` 的行为可以按下面的规则理解：

```text
Some(&T) --> 克隆这个 T --> Some(T)
None     ----------------> None
```

它没有把原来的 `T` 移走。两个方法的区别是操作的层次，而不是“clone 永远浅拷贝，cloned 永远深拷贝”。
如果内部类型是 `Arc<T>`，`cloned()` 仍调用 `Arc` 的克隆规则，共享底层对象；如果是 `String`，则复制文本。

### 示例二：原值销毁后，拥有的副本仍能使用

下面也是可独立运行的教学示例：

```rust
fn main() {
    let saved: Option<String> = {
        let original = String::from("text/event-stream");
        let borrowed: Option<&String> = Some(&original);

        borrowed.cloned()
    }; // original 在这里销毁，saved 拥有自己的 String。

    assert_eq!(saved.as_deref(), Some("text/event-stream"));
    println!("保存的值：{saved:?}");

    let missing: Option<&String> = None;
    let saved_missing: Option<String> = missing.cloned();
    assert!(saved_missing.is_none());
}
```

若将 `borrowed.cloned()` 改成 `borrowed.clone()`，结果仍是 `Option<&String>`，就不能在原字符串销毁后继续使用。
本例显式标注了 `Option<String>`，直接替换时会先遇到类型不匹配；即使改掉类型标注，引用仍不能逃出原值的有效范围。

## 阶段 11：先保存响应头再移动上游响应

先读 [SSE 逐块转发](../../walkthroughs/sse-scheduling.md#阶段-11-02不收集完整响应是什么意思)。以下摘自 [网关 chat_stream](../../../../src/gateway.rs)，仅展示响应头保存与流的交接：

```rust
let content_type = upstream.headers().get(CONTENT_TYPE).cloned();
let stream = upstream.bytes_stream();
```

| 写法 | 类型 | 能否不再借用 upstream 中的响应头 |
|---|---|---|
| `.get(CONTENT_TYPE)` | `Option<&HeaderValue>` | 否 |
| `.get(CONTENT_TYPE).clone()` | `Option<&HeaderValue>` | 否，复制的是引用 |
| `.get(CONTENT_TYPE).cloned()` | `Option<HeaderValue>` | 是，得到自己拥有的响应头值 |

```text
上游响应拥有响应头
       |
       | get：借用响应头
       v
Option<&HeaderValue>
       |
       | cloned：克隆引用指向的值
       v
Option<HeaderValue> --保存到局部变量，随后放入下游响应

上游响应 --bytes_stream() 消费--> 正文流 --交给--> 下游 Body
```

如果保留的只是引用，却在移动 `upstream` 后继续使用它，借用检查会拒绝，因为引用仍依赖原拥有者中的值。
`HeaderValue` 的克隆可能共享内部字节存储；“得到拥有的值”保证不再借用 `upstream`，不保证底层内存被完整复制。

自查：先写出表达式的接收者类型，再写调用后的类型，最后判断是否仍然依赖原值的存活时间。
