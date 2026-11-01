# 毒锁：知道状态可能不完整

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**持锁代码 panic 后，毒锁提示可能破坏不变量；拿回数据不等于修复业务状态。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Mutex 与 Guard：锁何时释放](mutex-and-guards.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/circuit_breaker/gate.rs](../../../../src/circuit_breaker/gate.rs)：`CircuitGate::lock_state` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `lock()` 为什么返回 `Result`

```rust
state.failures_remaining.lock()
```

返回类型可以展开为：

```rust
Result<
    MutexGuard<'_, usize>,
    PoisonError<MutexGuard<'_, usize>>,
>
```

也可以写成标准库的类型别名：

```rust
LockResult<MutexGuard<'_, usize>>
```

两个分支是：

```text
Ok(MutexGuard)          正常取得锁
Err(PoisonError)        取得锁，但 Mutex 已被标记为中毒
```

因此需要从 `Result` 中取出 Guard：

```rust
let mut remaining = match state.failures_remaining.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
};
```

这里的 `guard` 并没有被忽略。`match` 是表达式，每个分支都会产生最终赋给 `remaining` 的值：

```text
Ok 分支  ──> guard ───────────────┐
                                  ├─> remaining: MutexGuard<'_, usize>
Err 分支 ──> poisoned.into_inner()┘
```

这与 [`Result`、`?` 与错误传播](../errors/question-mark.md) 中的普通
`Result<T, E>` 是同一个模型，只是这里的 `T` 是 `MutexGuard`。

## 什么是毒锁

线程持有 `std::sync::Mutex` 时发生 panic，Mutex 会被标记为 poisoned：

```text
线程取得 MutexGuard
        │
        v
修改共享数据时 panic
        │
        v
MutexGuard 被 drop，锁被释放
        │
        v
Mutex 被标记为 poisoned
```

“中毒”不表示锁永远无法打开，而是在提醒调用者：

> 上一次修改可能只做了一半，内部数据的一致性需要重新判断。

下一次 `lock()` 返回 `Err(PoisonError)`。错误内部仍然携带取得的 Guard：

```rust
Err(poisoned) => poisoned.into_inner()
```

`into_inner()` 会消耗 `PoisonError`，取出其中的 `MutexGuard`。

```text
PoisonError<MutexGuard<T>>
           │ into_inner()
           v
      MutexGuard<T>
```

是否应该恢复取决于数据：

| 数据 | 常见选择 |
|---|---|
| 简单计数器 | 可以检查后继续使用 |
| 多字段且存在一致性约束的业务状态 | 先校验或重建，不能盲目继续 |

所以 `into_inner()` 不是“毒锁一定可以忽略”，而是显式选择继续检查或使用内部数据。

## 自查与关联

into_inner 与 clear_poison 是否等同于恢复正确状态？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
