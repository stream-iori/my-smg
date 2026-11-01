# ??：逐层拆开 Result 与 Option

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**两个问号分别作用于两层结果，每一层失败都会提前返回。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [? 与 ??：成功继续，失败提前返回](question-mark.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`timeout 的两层结果` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 连续两个问号：逐层处理嵌套结果

**`??` 不是独立运算符，而是连续使用两次 `?`，每次处理一层。**

## Result 套 Result：`handle.await??`

假设 Tokio 任务本身返回 `Result<T, BusinessError>`，等待任务后就有两层结果：

```text
handle.await
    ↓
Result<Result<T, BusinessError>, JoinError>
│      │
│      └── 第二个 ?：处理任务内部的业务错误
└───────── 第一个 ?：处理任务错误，例如 panic 或取消
```

调用片段可分成两步理解：

```rust
let business_result = handle.await?; // 得到 Result<T, BusinessError>
let value = business_result?;        // 得到 T
```

合起来就是 `let value = handle.await??;`。要求当前函数返回的错误类型能够接收两类错误，
必要时通过 From 转换；不能因为写了两个问号就自动兼容任意错误类型。

| `handle.await` 的结果 | 第一个 `?` | 第二个 `?` | 最终行为 |
|---|---|---|---|
| `Ok(Ok(42))` | 得到 `Ok(42)` | 得到 42 | 继续执行 |
| `Ok(Err(业务错误))` | 得到 `Err(业务错误)` | 提前返回错误 | 后续不执行 |
| `Err(任务错误)` | 提前返回错误 | 不执行 | 后续不执行 |

`JoinHandle<T>` 等待后返回 `Result<T, JoinError>`；如果 T 本身也是 Result，就产生上述嵌套。
任务完成、取消与业务失败是不同层面的结果。参见 [Tokio：JoinHandle](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinHandle.html)。
另一种两层结果来自 timeout，见 [timeout 与两层 Result](../../libraries/tokio/time.md)。

## Option 套 Option：同样逐层处理

```rust
fn nested_number(value: Option<Option<i32>>) -> Option<i32> {
    let number = value??;
    Some(number)
}
```

| 输入 | 结果 |
|---|---|
| `Some(Some(42))` | `Some(42)` |
| `Some(None)` | 第二个 `?` 提前返回 None |
| `None` | 第一个 `?` 提前返回 None |

如果嵌套的是 `Result<Option<T>, E>`，则不能在普通返回 Result 或 Option 的函数中直接用 `??`
跨越这两种失败通道；需要先明确 None 应转换成什么错误，或者是否丢弃 Err 的错误信息。

## 自查与关联

Ok(Err(error)) 与 Err(error) 分别在哪一步返回？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
