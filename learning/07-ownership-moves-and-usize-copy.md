# Rust：当前实现中的所有权移动与 `usize` 复制

[返回学习入口](./index.md)

## 一句话

```text
String、Worker、Vec<T> 默认会发生所有权移动；
usize 实现了 Copy，按值使用时复制一个新值，原变量仍然有效；
&T 和 &mut T 借用数据，不取得被借用数据的所有权。
```

“移动”描述的是 Rust 的所有权变化，不等于运行时一定逐字节搬运内存。编译器可能优化实际
机器操作，但所有权和变量可用性的语言规则不变。

## 当前代码中的所有权移动

### 1. `String` 移入 `Worker::new`

```rust
let worker = Worker::new(String::from("worker-a"));
```

`String::from` 创建一个拥有字符串缓冲区的 `String`。`Worker::new` 的参数按值接收：

```rust
pub fn new(id: String) -> Self
```

因此调用时，`String` 的所有权进入参数 `id`。如果先保存变量，移动会更明显：

```rust
let id = String::from("worker-a");
let worker = Worker::new(id);

// 这里不能再按值使用 id，因为所有权已经移动给 Worker::new。
```

### 2. 参数 `id` 移入结构体字段

构造节点时：

```rust
Self {
    id,
    status: HealthStatus::Healthy,
}
```

字段简写 `id` 相当于 `id: id`。右侧参数 `id` 的所有权被移动进新建的 `Worker.id` 字段。

```text
调用者的 String
       │ move
       ▼
new 的参数 id
       │ move
       ▼
Worker.id
```

函数返回后，创建出的 `Worker` 拥有这个 `String`。

### 3. `Worker` 移入 `Vec`

```rust
let workers = vec![worker_a, worker_b, worker_c];
```

当前 `Worker` 没有实现 `Copy`，所以三个节点的所有权都会移动进 `Vec<Worker>`。从此由
`workers` 拥有这些节点，原来的 `worker_a` 等变量不能继续使用。

```text
worker_a ─┐
worker_b ─┼─ move ─> Vec<Worker>
worker_c ─┘
```

当 `workers` 离开作用域时，`Vec` 会依次释放其拥有的 `Worker`，节点再释放各自拥有的
`String`。

### 4. 新状态移入 `Worker.status`

```rust
worker.set_status(HealthStatus::Unhealthy);
```

方法按值接收状态：

```rust
pub fn set_status(&mut self, status: HealthStatus) {
    self.status = status;
}
```

当前 `HealthStatus` 只派生了 `Debug`、`PartialEq` 和 `Eq`，没有实现 `Copy`。因此参数
`status` 被移动进 `self.status` 字段，字段原来的状态被替换并丢弃。

虽然这个枚举目前没有携带堆资源，所有权规则仍然按“非 `Copy` 类型发生移动”理解。

### 5. 返回 `Vec<usize>` 时转移所有权

```rust
pub fn healthy_worker_indices(workers: &[Worker]) -> Vec<usize> {
    let mut indices = Vec::new();
    // 填充 indices
    indices
}
```

`Vec<usize>` 拥有自己的缓冲区。最后一个表达式 `indices` 把这个 `Vec` 的所有权返回给调用者：

```text
函数局部变量 indices
          │ move / return
          ▼
调用者取得 Vec<usize> 的所有权
```

编译器通常会消除不必要的实际内存搬运，但从 Rust 语义看，返回后所有者已经变成调用者。

## 哪些地方只是借用

### `select(&workers)`

```rust
policy.select(&workers)
```

`&workers` 创建共享借用，并可转换为 `&[Worker]`。函数只能在借用有效期间读取节点，没有取得
`Vec` 或其中 `Worker` 的所有权。

调用结束后，借用结束，测试函数仍然拥有并可以继续使用 `workers`。

### `status(&self)`

```rust
pub fn status(&self) -> &HealthStatus
```

`&self` 借用当前 `Worker`，返回的 `&HealthStatus` 又借用了其中的状态。没有把状态从
`Worker` 中取走。

### `set_status(&mut self)`

`&mut self` 是对 `Worker` 的独占可变借用。方法可以修改节点，但仍不拥有整个 `Worker`；调用
结束后，节点所有权仍属于原来的变量或 `Vec`。

## `usize` 为什么是复制

`usize` 实现了 `Copy`。按值赋值、传参或返回时，会复制出一个新的整数值，原变量仍然可用：

```rust
let first: usize = 2;
let second = first;

assert_eq!(first, 2);
assert_eq!(second, 2);
```

这里 `second = first` 不会使 `first` 失效。

轮询代码从索引 `Vec` 中读取元素时也是如此：

```rust
let selected = healthy_indices[position];
```

索引表达式找到一个 `usize` 元素。因为 `usize: Copy`，`selected` 得到这个整数的副本，并不把
元素从 `healthy_indices` 中移走。

把 `selected` 放入 `Some` 并返回时，复制语义同样让小整数很容易按值传递：

```rust
Some(selected)
```

## `Copy` 与 `Clone` 的区别

| 能力 | 调用方式 | 原值是否仍可用 | 特点 |
|---|---|---:|---|
| `Copy` | 编译器隐式复制 | 是 | 适合 `usize` 等小型简单值 |
| `Clone` | 显式调用 `.clone()` | 是 | 可能执行更昂贵的复制 |
| move | 普通按值使用非 `Copy` 类型 | 否 | 所有权转移给新位置 |

`String`、`Vec<T>` 和当前的 `Worker` 都不是 `Copy`。它们按值使用时通常发生移动，而不是隐式
复制。若确实需要复制其内容，类型实现 `Clone` 后仍需显式调用 `.clone()`。

## 引用本身与引用背后的值

共享引用 `&T` 本身通常可以复制，但这不会复制它指向的 `T`：

```text
&Worker 被复制  -> 多个共享引用都指向同一个 Worker
Worker 被复制   -> 产生另一个独立 Worker
```

所以“传递 `&workers` 很轻量”不代表复制了整个节点列表；复制的只是用于访问同一份数据的引用
信息。

## 当前类型对照

| 类型或表达式 | 当前行为 | 原值是否还能使用 |
|---|---|---:|
| `String` 按值传入 `Worker::new` | move | 否 |
| `Worker` 放入 `vec![...]` | move | 否 |
| `HealthStatus` 按值传入 `set_status` | move | 否 |
| 返回 `Vec<usize>` | 所有权转移给调用者 | 函数中的局部变量结束 |
| `select(&workers)` | borrow | 是，借用结束后可用 |
| `worker.status()` | borrow | 是 |
| `usize` 赋值、传参或返回 | copy | 是 |

## 一句话记忆

```text
move：东西交给你，我不再使用。
borrow：东西还是我的，你临时使用。
Copy：给你复制一份，我这份仍然存在。
```
