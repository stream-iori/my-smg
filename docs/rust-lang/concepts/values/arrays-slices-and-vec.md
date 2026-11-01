# 数组、切片、Vec 与 usize

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Vec 拥有元素，切片借用一段连续元素，usize 用来表示长度和索引。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/policy.rs](../../../../src/policy.rs)：`healthy_worker_indices` |
| 阅读时机 | T03 读到基础自查；共享节点接口阶段 7 阅读。 |

以下是阶段 0～2 的同步接口形状，省略了函数体，不是当前源码的逐字摘录。

## `&[Worker]` 拆开看

| 部分 | 含义 |
|---|---|
| `Worker` | 单个元素的类型 |
| `[Worker]` | 一段连续 `Worker` 的切片；其长度不写在类型里 |
| `&[Worker]` | 对该切片的共享借用 |

切片本身可以理解为一幅“窗口”：它记录数据的起始位置和长度，但不拥有窗口背后的节点。

```text
workers: Vec<Worker>（拥有节点）
        │
        │ &workers
        ▼
&[Worker]（临时借用整段节点）
```

因此下面的函数不接管节点列表：

```rust
fn healthy_worker_indices(workers: &[Worker]) -> Vec<usize> {
    // 只读取 workers
}
```

调用结束后，原来的 `Vec<Worker>` 仍由调用者拥有，可以继续使用：

```rust
let workers = vec![/* Worker 值 */];
let indices = healthy_worker_indices(&workers);

// workers 仍然有效
println!("worker count: {}", workers.len());
```

## 为什么参数不写成 `Vec<Worker>`

如果按值接收：

```rust
fn healthy_worker_indices(workers: Vec<Worker>) -> Vec<usize>
```

调用时通常会把整个 `Vec` 的所有权移动进函数。调用者随后不能再使用原变量，除非函数把它
返回，或者调用前复制数据。

使用 `&[Worker]` 有两个优点：

1. 函数只借用数据，不取得所有权，也不需要克隆节点。
2. 参数不局限于 `Vec<Worker>`，还可以借用数组或某个局部范围。

```rust
healthy_worker_indices(&workers);       // 借用整个 Vec
healthy_worker_indices(&workers[1..3]); // 借用其中一段
```

`&[Worker]` 是共享借用，所以函数可以读取节点，但不能通过它修改节点。需要修改元素时，参数
类型才会是 `&mut [Worker]`。

## `[Worker]` 与 `[Worker; N]` 不同

```text
[Worker; 3]  长度为 3 的数组，长度属于类型的一部分
[Worker]     长度运行时才知道的切片，通常通过 &[Worker] 使用
Vec<Worker>  拥有堆上元素、长度可以增长的动态数组
```

切片类型 `[Worker]` 的大小在编译期不固定，因此一般不会单独按值使用，而是放在引用后面：
`&[Worker]` 或 `&mut [Worker]`。

## 为什么索引使用 `usize`

Rust 的切片和 `Vec` 使用 `usize` 表示长度与索引：

```rust
let index: usize = 0;
let worker = &workers[index];
```

因此“健康节点在原切片中的位置”自然适合返回 `Vec<usize>`。这里返回的是原始节点切片中的
索引，不是新的健康节点列表里的相对位置。

## 同步阶段的所有权关系

```rust
fn healthy_worker_indices(workers: &[Worker]) -> Vec<usize>
```

所有权关系是：

```text
调用者拥有 Vec<Worker>
        │
        ├─ 临时共享借用 ──> workers: &[Worker]
        │                         │
        │                         └─ 只读取并计算索引
        │
        └─ 调用结束后仍然拥有并可继续使用原 Vec

函数自己创建并返回 Vec<usize>
调用者取得这个索引 Vec 的所有权
```

边界场景中，没有健康节点时可以返回：

```rust
Vec::<usize>::new()
```

也可以在返回类型或比较上下文足够明确时写成：

```rust
Vec::new()
```

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 借用 workers 传给筛选函数后原 Vec 还能用吗？ |
| 解释 | 为什么索引使用 usize，且仍指向原集合？ |
| 边界 | 对长度不足 3 的列表借用 [1..3] 会怎样？ |

先写下判断和原因，再用自己的已有代码验证。

[struct、enum、impl 与 Self](struct-enum-and-impl.md) · [元组、元组结构体与 unit](tuples-and-unit.md) · [表达式、返回值与分号](expressions-and-semicolons.md)


## 阶段 7：共享节点接口

先读[Arc](../sharing/rc-and-arc.md)。当前参数是 &[Arc<Worker>]，外层仍借用切片，元素改成共享句柄。同步阶段不要求提前实现这种演进。
