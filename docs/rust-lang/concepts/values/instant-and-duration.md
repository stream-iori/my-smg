# Instant 与 Duration：时间点和时间长度

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**`Instant` 记录某个时刻，`Duration` 表示一段时间有多长。计算时间和等待时间是两件事。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | 基础部分可以直接阅读；认识变量与方法调用即可。 |
| 源码入口 | [src/token_bucket.rs](../../../../src/token_bucket.rs)：`refill_interval`、`last_refill`、`refill` |
| 阅读时机 | 阶段 5 起可读基础部分；令牌桶应用在阶段 10 阅读。源码存在不代表阶段已经验收。 |

## 三种类型分别回答什么问题

```text
开始时刻                                      结束时刻
Instant A ├────────── 经过 150ms ───────────────┤ Instant B
                         │
                   Duration：150ms
```

| 类型 | 表示什么 | 常见用途 |
|---|---|---|
| `std::time::Instant` | 单调时钟上的时间点 | 请求耗时、冷却起点、限流时间计算 |
| `std::time::Duration` | 非负的时间长度 | 100ms 的补充间隔、2 秒的超时额度 |
| `std::time::SystemTime` | 系统日历时钟上的时间点 | 与外部系统交换时间、计算 Unix 时间戳 |

`Instant::now()` 的含义是“记住此刻”。`Instant` 的内部表示不公开，不能直接读取为 Unix 时间戳或日期。
测量间隔用 `Instant`；需要和 Unix 起点比较时用 `SystemTime` 与 `UNIX_EPOCH`。Java 中，前者用途接近 `System.nanoTime()`，后者用途接近日历时间；Rust 的 `Instant` 与 Java 的 `java.time.Instant` 含义不同。

单调时钟适合计时，通常不随手动修改系统日期而倒退。它不承诺每次读取都不同，也不承诺纳秒级的实际精度；系统休眠是否计入经过时间取决于平台。标准库对平台异常的边界说明见 [Instant 官方文档](https://doc.rust-lang.org/std/time/struct.Instant.html)。

## 用类型分清时间点与时间长度

以下是独立概念示例，不涉及网关实现：

```rust
use std::time::{Duration, Instant};

let start = Instant::now();
let later = start + Duration::from_millis(150);
let elapsed = later.saturating_duration_since(start);
```

| 变量 | 类型 | 意思 |
|---|---|---|
| `start` | `Instant` | 起点 |
| `later` | `Instant` | 起点之后 150ms 的时刻 |
| `elapsed` | `Duration` | 两个时间点相隔 150ms |

**给时间点加 150ms，只是在计算另一个时间点，程序不会因此等待。**

```text
时间点 + 时间长度 → 另一个时间点
时间点 - 时间长度 → 另一个时间点
两个时间点的间隔 → 时间长度
```

| 表达式 | 结果 |
|---|---|
| `start + Duration::from_millis(150)` | `Instant`，起点之后的时刻 |
| `later.saturating_duration_since(start)` | `Duration`，本例为 150ms；若 later 早于 start，则为零 |
| `later.checked_duration_since(start)` | `Option<Duration>`；若 later 早于 start，则为 `None` |
| `start.elapsed()` | `Duration`，从 start 到执行此调用时经过了多久 |

`elapsed()` 会读取实际时钟；显式传入时间点，则能在测试里构造确定的时间关系。算术结果超出可表示范围时，应按需要使用 `checked_add`、`checked_sub` 等接口。

## Duration 的单位与构造

```text
1 秒 = 1,000 毫秒 = 1,000,000 微秒 = 1,000,000,000 纳秒
```

| 写法 | 时间长度 |
|---|---|
| `Duration::ZERO` | 零时长 |
| `Duration::from_secs(2)` | 2 秒 |
| `Duration::from_millis(150)` | 150 毫秒 |
| `Duration::from_micros(500)` | 500 微秒 |
| `Duration::from_nanos(100)` | 100 纳秒 |
| `Duration::new(1, 250_000_000)` | 1 秒加 250 毫秒，即 1.25 秒 |

`Duration::new(秒数, 纳秒部分)` 用整数秒与纳秒部分组成时间长度；它不会把 1.25 秒向上取整为 2 秒。见 [Duration 官方文档](https://doc.rust-lang.org/std/time/struct.Duration.html)。

对 1.25 秒调用以下方法，结果分别是：

| 方法 | 返回类型 | 结果 |
|---|---|---|
| `as_secs()` | `u64` | `1`，只取完整秒数 |
| `as_millis()` | `u128` | `1250`，总毫秒数 |
| `as_nanos()` | `u128` | `1_250_000_000`，总纳秒数 |
| `subsec_nanos()` | `u32` | `250_000_000`，不足一秒的纳秒部分 |

例如，把总纳秒数拆成 `Duration::new` 的两个参数：

```text
1_250_000_000 / 1_000_000_000 = 1             → 整秒
1_250_000_000 % 1_000_000_000 = 250_000_000   → 不足一秒的纳秒

合起来仍是 1.25 秒，数值没有改变。
```

## 基础自查

| 问题 | 要解释的关键点 |
|---|---|
| `start + 150ms` 是 `Instant` 还是 `Duration`？ | 时间点与时间长度不能混为一谈 |
| 给 start 加 150ms，会让程序等待吗？ | 计算与等待的区别 |
| 1.25 秒的 `as_secs()` 与 `as_nanos()` 分别是多少？ | 完整秒数、总纳秒数与纳秒部分的区别 |
| Rust 的 Instant 能直接当 Unix 时间戳吗？ | 单调时钟与日历时钟的用途 |

## 项目应用：阶段 10 的令牌桶时间计算

另需 [struct 与方法](struct-enum-and-impl.md)、[共享借用与可变借用](../ownership/borrowing.md)。本节只读已有代码的状态变化，不提供完整令牌桶实现。

### now 是时间点，“150ms”只是相对起点的简写

当前测试中的表达式：

```rust
let start = Instant::now();
let now = start + Duration::from_millis(150);
```

在图表里写“now 为 150ms”，完整含义是 **`now = start + 150ms`**，now 本身仍是 `Instant`。

| 字段或变量 | 类型 | 职责 |
|---|---|---|
| `tokens` | `usize` | 当前保存的令牌数量 |
| `refill_interval` | `Duration` | 每补一枚需要多长时间 |
| `last_refill` | `Instant` | 下一次计算经过时间时使用的起点 |
| `now` | `Instant` | 本次调用传入的时间点 |
| `elapsed` | `Duration` | 从 last_refill 到 now 经过了多久 |

### last_refill 记录从哪里继续算

当前源码摘录：

```rust
let elapsed = now.saturating_duration_since(self.last_refill);
```

假设初始令牌已经用完，每 100ms 补一枚，桶内空间足够：

| 代码里的步骤 | 代入的值 | 意思 |
|---|---|---|
| 旧 `last_refill` | `start` | 先从起点算 |
| 本次 `now` | `start + 150ms` | 在起点后 150ms 调用 |
| 计算 `elapsed` | `150ms` | 经过了多长时间 |
| 整除补充间隔 | `150 / 100 = 1` | 已经过一个完整间隔 |
| 对补充间隔取余 | `150 % 100 = 50ms` | 还剩多少时间没凑够下一枚 |

保留余数版本中的源码摘录：

```rust
self.last_refill = now - remainder;
```

```text
now         = start + 150ms
remainder   =          50ms
last_refill = (start + 150ms) - 50ms
            = start + 100ms
```

结果是一个时间点；它只更新桶里的记录，没有修改 now，也没有改变真实时间。

```text
start          start + 100ms      start + 150ms      start + 200ms
  |------------------|------------------|------------------|
               已处理到这里          本次调用           下一次补充点
               新 last_refill
```

下一次在 `start + 200ms` 调用，代码算的是 `200ms - 100ms = 100ms`，因此又能补一枚。

这里有两种不同的取余，不能混淆：

| 运算 | 目的 |
|---|---|
| 经过纳秒数 `%` 补充间隔纳秒数 | 得到尚未用于完整补充间隔的时间 |
| 余下纳秒数 `% 1_000_000_000` | 把这段时间拆成整秒与不足一秒的纳秒部分 |

补充间隔可以超过一秒，因此“余下的时间”也可能包含整秒。拆成秒与纳秒只是换表示方法，不需要向上取整。

### 保留余数与简化版本的取舍

当前源码保留余数。讨论过的简化方案是在补充后把 last_refill 更新为 now，丢弃不足一个间隔的余数；未满间隔时仍提前返回。

| 方案 | last_refill 的含义 | 行为 |
|---|---|---|
| 保留余数 | 已处理完的完整间隔边界 | 余下的时间继续用于下一次补充 |
| 补充后记录 now | 最近一次执行补充的调用时间 | 更容易读，但可能少补额度，限流更保守 |

例如每隔 150ms 检查一次、补充间隔为 100ms，且桶内空间足够：保留余数时前三次分别补 1、2、1 枚；简化版本分别补 1、1、1 枚。改变规则后，对应的余数测试也应由学习者调整。

### 为什么不需要后台定时补充

当前 `try_acquire` 先调用 `refill(now)`，再尝试扣一枚令牌。这是惰性补充：根据经过时间一次计算该补多少。

| 时刻 | 定时器补充 | 请求到来时补充 |
|---|---|---|
| 起点后 100ms、200ms、300ms | 分别加一枚 | 没有请求就不执行代码 |
| 起点后 350ms 才有请求 | 扣除已经存好的令牌 | 先算出三个完整间隔、补足额度，再扣一枚 |

本例假设桶最初已空且容量足够。两种方式都要限制容量；定时器受调度影响，若要避免漏补，也需要正确结算经过时间。
对“立即放行或拒绝”的组件，扣额度前结算即可。需要等待额度的队列还要考虑何时唤醒等待者，那是后续任务。

传入 now 也便于测试：直接构造 `start + Duration::from_millis(150)`，无需真实等待 150ms。只有调用 sleep 等等待接口，程序才会等待，见 [sleep 与 timeout](../../libraries/tokio/time.md)。

## 官方资料

- [std::time::Instant](https://doc.rust-lang.org/std/time/struct.Instant.html)：时间点、间隔计算和平台行为。
- [std::time::Duration](https://doc.rust-lang.org/std/time/struct.Duration.html)：时间长度、构造与单位换算。
- [std::time::SystemTime](https://doc.rust-lang.org/std/time/struct.SystemTime.html)：系统时钟与 Unix 时间起点。
