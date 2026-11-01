# 闭包与 Fn、FnMut、FnOnce

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**闭包保存环境；Fn 系列描述调用时如何访问这个环境，不是看函数是否会被调用一次。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [共享借用与可变借用](../ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`wait_until` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先理解一个调用

`my-smg` 的取消测试使用：

```rust
wait_until(Duration::from_secs(2), || worker.counter() == 1).await;
```

第二个参数不是已经算好的 bool，而是一段可再次执行的判断逻辑。

```text
传 bool：现在检查一次 → 等待函数只拿到 true 或 false
传闭包：保存检查办法 → 等待函数每轮重新读取计数
```

| 部分 | 含义 |
|---|---|
| `\|\|` | 闭包参数列表为空，不是逻辑或 |
| `worker.counter() == 1` | 闭包的执行内容，返回 bool |
| 外部的 worker | 闭包捕获的变量，本例共享借用它 |
| `.await` | 等待 wait_until 返回的 Future，不是等待 bool |

有参数时写成 `|value| ...`，有多条语句时可以使用块：`|| { ... }`。
闭包是一个有具体类型的值；编译器为不同闭包表达式生成不同的匿名类型。

## Fn、FnMut 与 FnOnce：区别在调用需要什么

| 能力 | 调用时的接收方式 | 常见行为 |
|---|---|---|
| `Fn` | `&self` | 只读取捕获状态，可重复调用 |
| `FnMut` | `&mut self` | 调用时可以修改捕获状态，可重复调用 |
| `FnOnce` | `self` | 消耗闭包，可以将捕获的非 Copy 值移出去 |

这些不是互斥标签：实现 Fn 的闭包也实现 FnMut 和 FnOnce；实现 FnMut 的闭包也实现 FnOnce。
FnOnce bound 只保证能按值调用一次，不表示所有满足它的闭包都只能调用一次。
参见 [标准库 FnMut](https://doc.rust-lang.org/std/ops/trait.FnMut.html)。

`wait_until` 的参数声明是：

```rust
mut condition: impl FnMut() -> bool
```

| 语法 | 职责 |
|---|---|
| `condition` | 参数变量名 |
| `mut` | 允许可变借用这个闭包值，以便调用 FnMut |
| `impl FnMut` | 参数接受一个满足约束的具体类型；不是 dyn 动态分发 |
| `()` | 调用闭包不传参数 |
| `-> bool` | 每次调用返回 bool |

用具名泛型表达，同一个约束可以写成 `F: FnMut() -> bool`，参数类型写 F。
泛型与 impl Trait 的基础见 [Trait 与分发](static-and-dynamic-dispatch.md)。

计数检查闭包只读取 worker，可以满足 Fn；因此也能传给要求 FnMut 的接口。
选择 FnMut 是允许更广的重复判断场景，不是要求每次必须修改状态。
例如下面的闭包修改自己捕获的普通变量，需要 FnMut 调用能力：

```rust
let mut attempts = 0;
let mut condition = || {
    attempts += 1;
    attempts >= 3
};
assert!(!condition()); // 第 1 次
assert!(!condition()); // 第 2 次
assert!(condition());  // 第 3 次
```

本例为展示闭包能力，不是建议用尝试次数代替真实状态判断。
Fn 系列描述闭包的调用方式，不描述其内部调用是否通过原子变量、锁等修改其他对象。

## 三个小例子对照

```rust
// Fn：读捕获的变量，反复调用不会取走 String。
let name = String::from("worker-a");
let name_len = || name.len();
assert_eq!(name_len(), 8);
assert_eq!(name_len(), 8);
```

```rust
// FnMut：修改捕获的普通变量，因此调用闭包的绑定需要 mut。
let mut count = 0;
let mut increment = || {
    count += 1;
    count
};
assert_eq!(increment(), 1);
assert_eq!(increment(), 2);
```

```rust
// 只满足 FnOnce：把捕获的 String 返回给调用方，闭包不再保有它。
let name = String::from("worker-a");
let take_name = move || name;
let owned_name = take_name();
assert_eq!(owned_name, "worker-a");
// take_name(); // 编译错误：第一次调用已消费闭包。
```

| 例子 | 捕获值的变化 | 对应能力 |
|---|---|---|
| name_len | 保留，只读 | Fn，也满足 FnMut、FnOnce |
| increment | 保留，但可修改 | FnMut，也满足 FnOnce |
| take_name | String 在调用时移出 | 只满足 FnOnce |

Fn 的绑定不需要 mut；FnMut 调用需要能可变借用闭包。
这里“修改捕获变量”指第二个例子里普通整数的赋值，不是把所有带修改操作的闭包都归为 FnMut。

## 自查与关联

FnOnce 是否代表运行时最多只能调用一次？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [关联类型：实现确定自己的 Output](associated-types.md)
