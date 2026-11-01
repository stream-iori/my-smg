# self、mut self、&self 与 &mut self

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**有没有 & 决定拥有还是借用，mut 决定绑定能否修改；这两件事分开看。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](borrowing.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`ManagedTask::cancel_and_join` |
| 阅读时机 | T02 读到基础自查；项目契约在阶段 3 后阅读；Arc 接收者与两个 guard 的对照在阶段 11 阅读。 |

## 先区分拥有、借用与修改权限

| 接收者 | 数据归谁 | 访问方式 |
|---|---|---|
| &self | 原拥有者 | 临时共享访问 |
| &mut self | 原拥有者 | 临时独占访问 |
| self | 按值交给方法 | 方法拥有收到的值；Copy 类型可以复制 |
| mut self | 与 self 一样按值接收 | 方法内部的 self 绑定可修改 |

Copy 是允许隐式复制的能力，整数通常可以复制，String 的按值交接则会移动。
调用 mut self 方法不要求原绑定为 mut：交出值后，方法内部建立新的可变绑定。
let mut 控制绑定可变性；&mut 表达一次独占借用。内部可变性在阶段 7 再学。

## &self 是 self 参数的简写

**不能只看有没有写出单词 `self`，要看整个接收者类型。**

| 常见写法 | 展开后的写法 | 含义 |
|---|---|---|
| `&self` | `self: &Self` | 共享借用当前对象 |
| `&mut self` | `self: &mut Self` | 独占可变借用当前对象 |
| `self` | `self: Self` | 按值接收当前对象 |

`Self` 表示当前实现的类型。在 `impl Worker` 中，`Self` 就是 `Worker`。
`self` 是方法的接收者参数名，`Self` 是类型，两者不能混为一谈。

## 点号调用会自动借用接收者

下面是可独立运行的教学示例，只观察调用方式：

```rust
struct Counter {
    value: usize,
}

impl Counter {
    fn value(&self) -> usize {
        self.value
    }
}

fn read_counter(counter: &Counter) -> usize {
    counter.value
}

fn main() {
    let counter = Counter { value: 3 };

    assert_eq!(counter.value(), 3);         // 方法调用自动借用
    assert_eq!(Counter::value(&counter), 3); // 显式传入接收者
    assert_eq!(read_counter(&counter), 3);   // 普通参数显式借用

    assert_eq!(counter.value, 3); // counter 没有被移动，仍然可用
}
```

```text
变量 counter 的类型：Counter
            |
            | counter.value() 根据接收者要求自动借用
            v
传入方法的 self 类型：&Counter
```

调用时临时借用不改变变量声明的类型，也不会复制整个对象。
点号方法调用还能进行自动解引用来查找方法，见 [Deref 与方法调用](../sharing/deref-and-guards.md#方法调用中的自动解引用与自动借用)。
这些接收者调整不代表任意普通函数参数都能省略 `&`；例如上面的 `read_counter(&counter)` 仍需显式借用。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 调用 mut self 方法前，原绑定必须声明为 mut 吗？ |
| 解释 | & 与 mut 在接收者中各解决什么问题？ |
| 解释 | counter 是 Counter，为什么可以调用接收 &self 的方法？ |
| 边界 | 非 Copy 值交给消费 self 的方法后还能再次使用吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

## 阶段 3 后：项目中的接收者契约

先读[trait](../types/traits-and-bounds.md)；任务与许可在阶段 8～10 阅读。

## 从项目三个签名看起


| 项目签名节选 | 接收者 | 作用 |
|---|---|---|
| Worker::id(&self) -> &str | &self | 只借用节点，返回文本视图 |
| Policy::select(&mut self, ...) -> Option<usize> | &mut self | 借用策略并推进轮询状态 |
| ManagedTask::cancel_and_join(mut self) | mut self | 接管管理对象，并修改其 Option 字段 |
| CircuitPermit::success(mut self, ...) | mut self | 消费许可，禁止原凭证被重复上报 |

这些是签名摘录，省略参数和函数体。没有 & 的 self 接管值；mut 只说明这个局部绑定可修改。

## 为什么共同签名使用 `&mut self`

当前 Policy::select 的 workers 参数为 `&[Arc<Worker>]`。下面去掉 Arc 的接口是同步阶段的简化示意，只比较 self 的借用方式。

`RoundRobin` 选择后必须修改计数器，因此它需要：

```rust
fn select(&mut self, workers: &[Worker]) -> Option<usize>
```

`FirstHealthy` 本身没有状态，本来只需 `&self`。但是实现同一个 trait 时，方法签名必须一致，
所以它也接收 `&mut self`。

这说明接收者类型也是契约的一部分：

```text
&self      调用期间共享借用实现
&mut self  调用期间独占可变借用实现
self       调用会取得实现的所有权
```

如果未来要让多个任务并发调用一个需要 `&mut self` 的策略，仅仅加 `Arc` 不够；还需要重新设计
状态，或使用 `Mutex` 等同步机制。`Arc<T>` 只解决共享所有权，不自动提供可变访问。

所有权与借用基础见[所有权、转移与借用](move-and-partial-move.md)。

## 阶段 11：`&self` 与 `self: &Arc<Self>`

先读 [Arc 句柄、句柄引用与目标引用](../sharing/rc-and-arc.md#arctarct-与-t-要分开)。下面是当前 [Worker](../../../../src/worker.rs) 的签名摘录，省略方法体：

| 接收者写法（位于 impl Worker 中） | 实际传入类型 | 方法能得到什么 |
|---|---|---|
| `&self` | `&Worker` | 借用 Worker，适合创建借用型 guard |
| `self: &Arc<Self>` | `&Arc<Worker>` | 借用现有 Arc 句柄，可以克隆句柄创建拥有型 guard |
| `self: Arc<Self>` | `Arc<Worker>` | 按值接收一个 Arc 句柄；本项目的创建方法没有采用这个接收者 |

`Arc<Self>` 和 `&Arc<Self>` 是 Rust 支持的接收者形式。它们可以写在 `impl Worker` 中，不要求方法定义在 `impl Arc` 中；这不意味着任意包装类型都能作为稳定 Rust 的接收者。

```rust
pub fn begin_request(&self) -> InFlightGuard<'_>
pub fn begin_request_owned(self: &Arc<Self>) -> OwnedInFlightGuard
```

两个方法都共享借用接收者。第二个方法中的 `self:` 只是显式标注接收者类型，不能据此认定它移动了调用者的 Arc。

假设 `worker` 是 `Arc<Worker>`，以下片段对照同一次调用的写法，不是要求连续创建三个请求：

```rust
worker.begin_request_owned();
(&worker).begin_request_owned();
Worker::begin_request_owned(&worker);
```

```text
worker：Arc<Worker>
    |
    | 点号调用时自动借用
    v
self：&Arc<Worker>
    |
    | 方法内部 Arc::clone(self)
    v
新的 Arc<Worker> --> 放入 OwnedInFlightGuard
```

接收者只在调用期间借用外部句柄，返回值拥有自己的句柄。外部 `worker` 仍能继续使用。
只拿到 `&Worker`，不能直接调用 `Arc::clone` 得到原来的共享句柄；借用目标与借用句柄是不同能力。

对照调用链：

| 调用 | 根据接收者需要进行的调整 | 返回值关系 |
|---|---|---|
| `worker.begin_request()` | 借助 Arc 的解引用能力借用目标 Worker | guard 中的引用继续依赖这次借用 |
| `worker.begin_request_owned()` | 借用 Arc 句柄 | guard 保存克隆得到的 Arc，不依赖输入借用继续存活 |

签名中的返回值约束见 [两种 guard 的生命周期](lifetimes.md#阶段-11借用型-guard-与拥有型-guard)。
创建、移动和销毁的完整流程见 [Worker 导读](../../walkthroughs/worker-lifecycle.md#阶段-11拥有型-guard-独立保存节点句柄)。

自查：`Arc::new` 得到的变量类型是什么？调用 `begin_request_owned` 时，方法收到的又是什么类型？
