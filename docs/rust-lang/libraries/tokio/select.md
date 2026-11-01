# select!：等待多个事件，处理先就绪的分支

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**select! 在当前任务中推进多个 Future；一个 Future 就绪且结果匹配模式后，执行该分支并结束这次 select!。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Future 与 await](../../concepts/async/future-and-await.md) · [JoinHandle](join-handle.md) · [match 与守卫](../../concepts/patterns/match-and-guards.md) |
| 源码入口 | [src/shutdown.rs](../../../../src/shutdown.rs)：`wait_for_shutdown / run`；[HTTP 生命周期测试](../../../../src/gateway/tests/lifecycle/http.rs) |
| 阅读时机 | 阶段 12；以下按 Cargo.lock 当前锁定的 Tokio 1.53.1 解释，不代表停机实现已验收。 |

## 为什么不能依次 await？

```text
依次 await：等服务器结束 → 再等停机信号
                            ↑ 服务器不结束，就到不了这里

select!：    ┌─ 等服务器结束 ─┐
            └─ 等停机信号 ──┴─ 任一分支就绪，进入相应处理
```

select! 适用于“任一事件发生就做决定”；join! 适用于“全部异步操作都完成后再继续”。它们不会自动为每个分支创建任务。与 Java CompletableFuture.anyOf 可作职责类比，但要额外考虑 Rust 对落选 Future 的销毁。

## 语法与返回值

```rust
// 独立教学函数；模拟业务尚未结束，但取消通知已经就绪。
async fn choose_event() -> &'static str {
    tokio::select! {
        value = std::future::pending::<u64>() => {
            let _ = value;
            "business_finished"
        }
        _ = std::future::ready(()) => {
            "cancel_requested"
        }
    }
}
```

| 组成部分 | 含义 |
|---|---|
| `value = future` | 推进 Future，结果就绪后用模式匹配并绑定变量 |
| `_ = future` | 接受任意返回值并忽略，包括 Err |
| `=> { ... }` | 获胜分支的处理代码 |
| 分支最后的表达式 | 整个 select! 的返回值；各分支返回类型须兼容 |

分支中不用额外写 await，宏负责等待。上例必选第二分支，因为 pending 永不就绪。真实 I/O 若同时就绪，不应假设源码中上面的分支一定获胜；默认会随机选择检查起点来改善公平性。`biased;` 改成从上到下检查，但要由调用者避免后面的分支饥饿。

## 并发推进，不是多线程同时执行

```text
当前任务 poll A → A 返回 Pending
       poll B → B 返回 Pending
       暂停，等某个事件唤醒
       再次推进 → 某分支就绪 → 执行处理代码
```

这是概念流程，不保证固定检查顺序。分支在同一个当前任务中被推进；若其中有长时间同步阻塞，其他分支也无法推进。分支若等待的是 JoinHandle，句柄对应的工作已由 spawn 独立调度；独立任务来自 spawn，而不是 select!。见 [Tokio select! 文档](https://docs.rs/tokio/1.53.1/tokio/macro.select.html)。

## 落选意味着取消什么？

| 分支等的是什么 | 另一分支获胜后的行为 |
|---|---|
| select! 自己拥有的普通 Future | 该 Future 被销毁，不再推进；已发生的远端副作用不会撤销 |
| 按值移入的 JoinHandle | 句柄被丢弃，已 spawn 的任务仍可能继续运行 |
| `&mut JoinHandle` | 当前借用结束；外面的句柄保留，后续可 await 或 abort |
| `&mut oneshot::Receiver` | 当前借用结束；接收端保留，之后仍可接收 |

项目在 run 中使用 `result = &mut server_task`，是为了让停机信号先到时，后面还能等待服务器排空。若服务器分支已获胜并取到了结果，则保存该结果，不再次 await 已完成的句柄。

注意三者不同：**停止等待句柄、取消已 spawn 的任务、撤销远端工作**。select! 不会自动替你完成后两项。关联：[取消](cancellation.md)、[oneshot](oneshot.md)。

## 阶段 12：临时借用 JoinHandle，保留后续清理能力

项目中的分支是 Rust 语法：

```rust
result = &mut server_task => {
    Some(result)
}
```

这里有三层对象，不能混为一谈：

```text
服务器任务：spawn 后已独立运行，负责 HTTP 工作
    ↑
server_task：JoinHandle，拥有等待结果和请求取消的能力
    ↑
&mut server_task：当前 select! 临时借用这个句柄来等待
```

`result = ...` 是 select! 的分支绑定语法，不是普通赋值；右侧结果就绪后，左侧变量 result 接住结果。可以近似理解为等待 `(&mut server_task).await`，但 select! 同时还推进其他分支。

### 按值移入与临时借用

| 分支写法 | 谁拥有句柄 | 如果停机信号先就绪 |
|---|---|---|
| `result = server_task` | 句柄移动进 select! | 落选时句柄销毁，任务仍可能运行；外面不能再通过这个变量等待 |
| `result = &mut server_task` | 外面仍拥有句柄，select! 只借用 | 借用结束，外面继续持有句柄，可以等待排空或请求取消 |

这不是复制句柄，也不是重新启动服务器。设计目的在于：**这次等待可以结束，但负责后续清理的管理者不能失去句柄。**

Future 的推进需要可变访问等待状态，例如登记唤醒信息和取出结果，因此这里使用 `&mut`；共享借用 `&JoinHandle` 不能代替它。局部变量必须是 `let mut server_task`，才能借出 `&mut`。借用期间不能再同时独占使用同一个句柄；select! 结束、当前借用不再使用后，才继续下一次等待。这符合普通借用规则，见[共享借用与可变借用](../../concepts/ownership/borrowing.md)。

### 为什么保存 Some(result)，另一个分支返回 None？

```text
                        select!
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
         服务器先结束               停机信号先到
              │                         │
        已取出任务结果             只结束这次借用等待
              │                         │
        Some(result)                   None
              │                         │
        检查保存的结果             继续 await 原句柄
```

Some 保存的是任务结果，不是句柄。当前服务器任务返回 `io::Result<()>`，所以 result 是 `Result<io::Result<()>, JoinError>`；再包一层 Option，表示是否已经在 select! 中取到结果。

**句柄保留下来，不代表结果可以取两次。** 服务器分支获胜后应使用保存的 result，不能再次 await 已完成并已取出结果的 JoinHandle。因此 shutdown::run 后面分成 Some 与 None 两条路径；这不仅是语法需要，也是在记录清理流程的状态。

### 可运行的小例子：先处理通知，再等原任务

下面用 ready 模拟已到达的控制通知，用 oneshot 保证任务此时尚未结束，使分支选择不依赖时间碰运气：

```rust
async fn retain_handle_until_cleanup() {
    let (finish_tx, finish_rx) = tokio::sync::oneshot::channel::<u64>();
    let mut task = tokio::spawn(async move {
        finish_rx.await.expect("completion value should arrive")
    });

    let result = tokio::select! {
        result = &mut task => Some(result),
        _ = std::future::ready(()) => None,
    };

    // 发送端还在，且没发送；任务尚未结束，所以通知分支获胜。
    assert!(result.is_none());
    finish_tx.send(42).expect("worker should still be waiting");

    let value = match result {
        Some(result) => result.expect("task should finish normally"),
        None => task.await.expect("task should finish normally"),
    };
    assert_eq!(value, 42);
}
```

通知先到并没有取消 task；临时借用结束后，我们仍然能用原句柄取得 42。生产代码若实际需要取消，必须显式 abort，并等待确认。具体 API 契约见 [JoinHandle 的取消安全性](https://docs.rs/tokio/1.53.1/tokio/task/struct.JoinHandle.html#cancel-safety)。

## 模式不匹配、else 与循环

| 写法 | 要注意什么 |
|---|---|
| `Ok(value) = operation()` | 若 operation 返回 Err，该分支在本次 select! 中禁用；错误不会自动传播 |
| `value = operation()` | 接住整个结果，再显式处理 Ok/Err，通常更容易观察错误 |
| `..., if condition` | condition 为 false 时不 poll 该分支，但 Future 表达式仍会被求值 |
| `else => ...` | 所有分支都被禁用时执行；不是“所有分支暂时 Pending”时执行 |

所有分支禁用且没有 else 会 panic。若在循环内重新创建 sleep，其他分支每次获胜都可能重置计时器；固定期限应保存 Future 并正确 pin，或使用绝对截止时间。某些 read_exact 等操作被取消后可能丢失局部进度，应检查相应 API 的 cancellation safety，而不能只看代码能否编译。

## 自查

1. 为什么 `server_task.await` 后再等待停机信号不合适？
2. 停机信号分支获胜，会自动 abort 服务器任务吗？
3. 所有分支 Pending 时，else 会立即执行吗？
