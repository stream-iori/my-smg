# Move 与部分移动

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**移动转移拥有值；只移走一个字段后，整体暂时不再完整。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md) |
| 源码入口 | [src/worker.rs](../../../../src/worker.rs)：`Worker::new` |
| 阅读时机 | 阶段 1～2 读到基础自查；可选字段、配置和捕获延后阅读。 |

## 所有权解决什么问题

以 `String` 为例，它在栈上保存指针、长度和容量，真正的字符存放在堆上：

```rust
let text = String::from("hello");
```

```text
栈                         堆
text
├─ ptr  -----------------> "hello"
├─ len = 5
└─ capacity = 5
```

变量 `text` 拥有这个 `String`。当 `text` 离开作用域时，Rust 自动调用 `drop`，释放堆内存：

```rust
{
    let text = String::from("hello");
    println!("{text}");
} // text 离开作用域，String 在这里被释放
```

编译器需要始终知道谁负责释放资源，这就是所有权规则的目的。

## move：转移所有权，不复制数据

```rust
let first = String::from("hello");
let second = first;

println!("{second}");
// println!("{first}"); // 编译错误：first 已经被 move
```

`let second = first` 没有复制堆上的字符串，只把所有权交给 `second`：

```text
转移前：first  ──拥有──> String("hello")

转移后：first  ──失效
        second ──拥有──> 原来的 String("hello")
```

如果 `first` 仍然有效，两个变量就会在离开作用域时释放同一块内存。Rust 让旧变量失效，从编译期
避免 double free。

## move 不只发生在赋值时

按值传参也会转移所有权：

```rust
fn consume(text: String) {
    println!("{text}");
} // text 在函数结束时释放

let message = String::from("capture payment");
consume(message);

// println!("{message}"); // 编译错误：所有权已经传给 consume
```

放入容器同样会 move：

```rust
let message = String::from("capture payment");
let mut messages = Vec::new();

messages.push(message);
// println!("{message}"); // 编译错误：String 现在由 Vec 拥有
```

## 函数返回值也能转移所有权

```rust
fn create_command() -> String {
    let command = String::from("capture payment");
    command
}

let command = create_command();
```

关系是：

```text
create_command 中的 command
              │ move
              ▼
调用方的 command
```

返回 `String` 不会因为局部变量离开作用域而产生悬垂指针；所有权已经移动给调用方。现代 Rust
还会使用返回值优化，通常不需要为了性能改成复杂的输出参数。

如果函数消费一个值后还要把它交还，也可以返回它：

```rust
fn normalize(mut text: String) -> String {
    text.make_ascii_lowercase();
    text
}

let name = String::from("PAYMENT");
let name = normalize(name);
```

但函数只需要临时使用值时，借用通常更合适。

## `use of moved value`

```rust
let text = String::from("hello");
consume(text);
// consume(text); // value used here after move
```

检查真实意图：

```text
函数只读取？       -> 参数改为 &str 或 &T
原变量不再需要？   -> 保持 move
双方都需要所有权？ -> 明确调用 clone()
```

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | String 传给 consume 后还能直接使用原变量吗？ |
| 解释 | 返回局部 String 的拥有值，与返回它的引用有什么区别？ |
| 边界 | String 移进 Vec 后，还能从原变量读取同一文本吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

[共享借用与可变借用](borrowing.md) · [self、mut self、&self 与 &mut self](method-receivers.md) · [Copy 与 Clone：复制值还是共享句柄](copy-and-clone.md)


## 阶段 3：结构体字段的部分移动

本节派生 Debug 的语法先见[derive](../types/derive-and-default.md)。

## 结构体字段也可以发生部分 move

```rust
#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".to_owned(),
    age: 20,
};

let name = user.name; // String 被 move
println!("{}", user.age); // age 是 Copy，仍可使用

// println!("{:?}", user); // 整个 user 已经不完整，不能再整体使用
```

这称为 partial move。常见解决方式是借用字段：

```rust
let name = &user.name;
println!("{name}");
println!("{}", user.age);
```

## 阶段 3 后：取走可选字段

先读[Option](../errors/option-and-result.md)和[借用与取值](../errors/option-access-and-take.md)。

或者在确实要取走可选字段时使用 `Option::take()`：

```rust
struct Payment {
    channel_transaction_id: Option<String>,
}

let mut payment = Payment {
    channel_transaction_id: Some("tx-100".to_owned()),
};

let transaction_id = payment.channel_transaction_id.take();

assert_eq!(transaction_id.as_deref(), Some("tx-100"));
assert!(payment.channel_transaction_id.is_none());
```

`take()` 把 `Some(String)` 移出来，同时在原位置留下 `None`，所以结构体仍然是完整可用的值。

## 阶段 4：配置中的所有权

先读[Result](../errors/option-and-result.md)与[问号](../errors/question-mark.md)。

## 所有权发生了什么

```rust
let config = GatewayConfig::from_json(input)?;
```

- `input: &str` 只是被借用；
- 成功时，`GatewayConfig` 从 `Ok` 中被取出并移动到局部变量 `config`；
- 失败时，错误值被移动到当前函数的返回值中；
- 配置字段是 `String`、`Vec` 等拥有型数据，因此成功结果不继续借用输入文本。



## 阶段 9：闭包与异步捕获

先读[闭包](../types/closures-and-fn.md)、[Future](../async/future-and-await.md)、[Arc](../sharing/rc-and-arc.md)。

闭包或异步任务使用 `move` 时，也会取得捕获值的所有权：

```rust
let request_id = String::from("req-100");

let task = move || {
    println!("{request_id}");
};

task();
// request_id 已经属于闭包
```

`move` 关键字表示闭包按值捕获；如果捕获的类型实现了 `Copy`，实际得到的是副本。

异步块同样需要区分捕获方式：`async { ... }` 可以推断借用或移动，`async move { ... }`
明确按值捕获。move 引用不延长被引用对象的存活时间，move Arc 也不自动 clone。
相关例子和 spawn 的生命周期要求见
[异步捕获：不写 move，也可能移动变量](../async/async-capture.md)。
