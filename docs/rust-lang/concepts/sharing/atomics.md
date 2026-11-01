# 原子变量与 Ordering：计数和跨数据同步

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**原子操作避免单次更新被拆开，Relaxed 不提供额外跨数据同步顺序。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md) · [Copy 与 Clone：复制值还是共享句柄](../ownership/copy-and-clone.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker.rs](../../../../src/worker.rs)：`fetch_add / fetch_sub` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 为什么 `fetch_add` 只需要 `&self`

普通字段和原子字段的区别：

| 字段 | 修改时通常需要 | 原因 |
|---|---|---|
| `usize` | `&mut self` | 普通写入要求独占访问 |
| `AtomicUsize` | `&self` | 类型自身保证并发读写安全 |

这叫**内部可变性**：外部只拿到共享引用 `&Worker`，但字段提供受控的修改方法。
它不是“所有 `&T` 都能随便修改”，也不是跳过 Rust 的安全规则；安全性由字段类型负责。

`AtomicUsize::fetch_add`、`fetch_sub` 和 `load` 的接收者都是 `&self`。因此包装它们的
`Worker::increment_count`、`decrement_count`、`counter` 也可以用 `&self`。
当前 `Worker::set_status(&self, ...)` 也使用原子字段 AtomicBool，因此同样只需共享引用；普通非内部可变字段的修改才通常需要独占借用。

## 三个操作分别做什么

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let count = AtomicUsize::new(0);
assert_eq!(count.load(Ordering::Relaxed), 0);

let old = count.fetch_add(1, Ordering::Relaxed);
assert_eq!(old, 0); // 返回修改前的值；此时 count 已经是 1

let old = count.fetch_sub(1, Ordering::Relaxed);
assert_eq!(old, 1); // 此时 count 回到 0
```

`load` 已经返回 `usize`；如果函数返回类型是 `usize`，不要在前面加 `&`，否则会变成
`&usize`。不需要旧值时，可在 `fetch_add` / `fetch_sub` 调用后加分号，丢弃返回值。

原子“加一”是不可分割的一次操作。多个请求同时加一，不会因为各自先读到同一个旧值而丢失更新。

## `Relaxed` 到底放松了什么

```text
Relaxed：仍保证这一个计数的读写是原子的
         不保证它与其他内存读写的先后关系
```

在途计数只用于统计，不用它的值来判断其他字段是否已经更新，所以这里可以使用
`Ordering::Relaxed`。如果把计数用作“其他数据已经准备好”的信号，不能直接照搬这个选择；
那需要重新设计并发同步关系。

## Ordering 的五种选择

Ordering 描述原子操作与其他内存操作如何建立同步关系。所有选择都保留该操作的原子性；它不是线程调度顺序、优先级，也不是让读操作“等待最新值”的开关。

| 选择 | 主要用途 | 常见场景 |
|---|---|---|
| Relaxed | 只需要原子更新这个值，不借它发布其他数据 | 请求总数、独立的数字 ID |
| Release | 发布此前的写入 | 生产者写好数据后设置 ready |
| Acquire | 读取到对应 Release 发布的值后，保证后续读取能观察到此前写入 | 消费者看到 ready 后读取数据 |
| AcqRel | 一次读改写操作同时承担获取和发布职责 | 需要双向同步的状态转换；当前计数不需要 |
| SeqCst | 提供相应 Acquire/Release 效果，并让 SeqCst 操作服从统一总顺序 | 算法确实需要多个原子操作的全局顺序时 |

Acquire/Release 必须形成同步关系；仅仅一边写了 Release、另一边写了 Acquire，不代表它们一定配对。下面的一次性例子中，消费者必须读到生产者写入的 true。

### 场景一：项目中的请求计数

[src/main.rs](../../../../src/main.rs) 的 REQUESTS_RECEIVED_TOTAL 是独立计数：中间件原子加一，指标接口原子读取。它不用于证明节点、配置或其他数据已经准备好，因此 Relaxed 足够。

```text
请求 A ── fetch_add(1, Relaxed) ──┐
请求 B ── fetch_add(1, Relaxed) ──┼─> 同一个计数器，无丢失更新
请求 C ── fetch_add(1, Relaxed) ──┘
```

并发处理过程中查询指标，得到的是一次采样；它不表示所有正在执行的加一都已完成。若要断言最终数量，需要先通过任务完成等机制确认工作已结束。ID 计数器也可用 Relaxed：它只负责分配数字，不负责确定请求谁先执行完；固定宽度整数溢出仍需另行考虑。

### 场景二：通过标志发布其他数据

下面是独立教学例子，不是项目的新功能。数据也使用原子类型，避免引入 unsafe；例子只发布一次。

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

let data = AtomicUsize::new(0);
let ready = AtomicBool::new(false);

std::thread::scope(|scope| {
    scope.spawn(|| {
        data.store(42, Ordering::Relaxed);
        ready.store(true, Ordering::Release);
    });

    scope.spawn(|| {
        while !ready.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        assert_eq!(data.load(Ordering::Relaxed), 42);
    });
});
```

```text
生产者：写 data=42 → Release 写 ready=true
                              │ 消费者读到该 true，建立同步
消费者：                 Acquire 读 ready → 读取 data=42
```

若 ready 的读写都改为 Relaxed，就不能靠这个标志保证 data 的结果一定为 42，即使在某台机器上多次运行都成功。实际工程优先考虑 channel、Mutex 或 OnceLock 等现成同步工具；这个循环只用于展示内存顺序，不适合作为异步任务中的等待方式。

### 哪些操作可以用哪些 Ordering？

| 操作 | 可使用 |
|---|---|
| load | Relaxed、Acquire、SeqCst |
| store | Relaxed、Release、SeqCst |
| fetch_add、swap 等读改写 | 五种均可，具体选择取决于同步职责 |

对读改写操作用 Acquire 时，写入部分是 Relaxed；用 Release 时，读取部分是 Relaxed；AcqRel 兼有两者。compare_exchange 失败时没有写入，因此失败顺序不能选 Release 或 AcqRel。

SeqCst 不会把多个原子变量变成一笔事务，也不会解决多字段快照不一致。不要只因为它“最强”就跳过算法设计；拿不准同步关系时，优先使用锁等更容易解释的机制。依据：[标准库 Ordering 文档](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)。

## 与 `Arc`、`Mutex` 的分工

| 类型 | 解决的问题 | 当前例子 |
|---|---|---|
| `Arc<T>` | 多个请求共同拥有状态 | 共享 `GatewayState` |
| `AtomicUsize` | 并发修改一个整数 | 每个 `Worker` 的在途计数 |
| `Mutex<T>` | 独占访问一组需要一起修改的数据 | 当前轮询策略的 `next` |

原子类型适合简单的独立数值操作。若必须同时检查、修改多个字段并保持整体一致，通常需要锁或
重新设计状态；把多个原子变量放在一起并不会自动产生一个“原子事务”。

`fetch_sub(1, ...)` 在计数为 `0` 时会下溢，不能无条件调用。当前练习先约定“加一后才减一”；
随后用 guard 的创建与销毁把两者配对，保证成功、失败和请求取消时都能归零。

## 自查与关联

fetch_add 返回增加前还是增加后的值？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
