# 共享借用与可变借用

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**借用只提供临时访问，不接管数据；共享读与独占修改不能冲突。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Move 与部分移动](move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：`Policy::select` |
| 阅读时机 | T02～T06 读到基础自查；策略契约在阶段 3。 |

## 借用 `&T`：暂时读取，不取得所有权

```rust
fn print_length(text: &String) {
    println!("{}", text.len());
}

let message = String::from("hello");

print_length(&message);
print_length(&message);
println!("{message}");
```

`&message` 创建共享引用：

```text
message ──拥有──> String("hello")
                    ▲
                    │ 借用
text: &String ------┘
```

函数只借来看，结束后所有权仍属于 `message`。

字符串参数通常优先写成 `&str`，这样既能接收 `String`，也能接收字符串字面量：

```rust
fn validate_order_no(order_no: &str) -> bool {
    !order_no.is_empty()
}

let owned = String::from("ORDER-100");

assert!(validate_order_no(&owned));
assert!(validate_order_no("ORDER-200"));
```

## 可变借用 `&mut T`：暂时独占并修改

```rust
fn mark_captured(status: &mut String) {
    status.clear();
    status.push_str("CAPTURED");
}

let mut status = String::from("AUTHORIZED");
mark_captured(&mut status);

assert_eq!(status, "CAPTURED");
```

`&mut status` 没有取得 `String` 的所有权，但在借用期间拥有独占访问权。

## 多个共享借用，或者一个可变借用

同一时刻可以有多个 `&T`：

```rust
let text = String::from("hello");

let first = &text;
let second = &text;

println!("{first} {second}");
```

但不能让共享借用和可变借用同时被使用：

```rust
let mut text = String::from("hello");

let reader = &text;
// let writer = &mut text; // 如果后面还使用 reader，这里会编译失败

println!("{reader}");
```

也不能同时使用两个可变借用：

```rust
let mut text = String::from("hello");

let first = &mut text;
// let second = &mut text; // first 后面仍会使用，因此编译失败

first.push('!');
```

规则可以记成：

```text
同一时刻：
任意多个只读者 &T
或者
一个写入者 &mut T
```

它在编译期避免数据竞争和迭代过程中修改容器等问题。

## `cannot borrow ... as mutable`

```rust
let text = String::from("hello");
// text.push('!'); // text 没有声明为 mut
```

变量需要声明为可变，并通过 `&mut` 传给修改者：

```rust
let mut text = String::from("hello");
text.push('!');
```

## `cannot borrow ... as mutable because it is also borrowed as immutable`

这表示一个共享引用后面仍会使用，却同时创建了可变引用。通常应缩短共享借用的使用范围，或者
调整代码顺序，不要立即用 `.clone()` 掩盖问题。

## 决策表

| 需求 | 参数或操作 |
|---|---|
| 函数只读取值 | `&T`；字符串常用 `&str` |
| 函数原地修改值 | `&mut T` |
| 函数接管值并负责后续处理 | `T`，发生 move |
| 函数消费后产生另一种值 | `self` / `T`，常用 `into_*` 命名 |
| 两个地方必须分别拥有值 | `.clone()` |
| 小型无资源值允许隐式复制 | 实现 `Copy` |

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | print_length 连续借用两次后，message 还能使用吗？ |
| 解释 | 只读访问与独占修改为什么要求不同的引用？ |
| 边界 | 共享引用后续仍会使用，能先修改原值吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

[Move 与部分移动](move-and-partial-move.md) · [self、mut self、&self 与 &mut self](method-receivers.md) · [Copy 与 Clone：复制值还是共享句柄](copy-and-clone.md)


## 阶段 3：策略中的借用契约

先读[trait](../types/traits-and-bounds.md)。

## 不能把 `&` 传给要求 `&mut` 的参数

trait 契约要求 `&mut self` 时，变量绑定和传参都必须允许可变借用：

```rust
let mut policy = RoundRobin::new();
select_with_policy(&mut policy, &workers);
```

## 阶段 12：借用任务句柄，不交出清理能力

`select!` 中的 `&mut server_task` 也是临时可变借用：管理者仍拥有 JoinHandle，当前只借它来等待。如果另一个分支先结束，借用结束，后面仍可等待服务器排空。它不会克隆句柄，也不会自动取消服务器任务。

完整分支流程与可运行例子统一见 [临时借用 JoinHandle](../../libraries/tokio/select.md#阶段-12临时借用-joinhandle保留后续清理能力)。本节是后续应用，不要求基础借用练习提前学习异步框架。
