# Send 与 Sync：哪些值能跨线程

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Send 表示可以转移到线程，Sync 表示共享引用可以跨线程；Arc 不会修复内层类型的线程安全。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Mutex 与 Guard：锁何时释放](mutex-and-guards.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`Mutex<Box<dyn Policy + Send>>` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 当前策略为什么只要求 Send

GatewayState 保存 Mutex<Box<dyn Policy + Send>>。多个请求共享 GatewayState，但一次只持锁访问一个策略。

```text
Arc<GatewayState> → Mutex → Box<dyn Policy + Send>
                    │
                 独占访问
```

Mutex 允许把受保护的 T 在持锁线程之间安全交接，因此这里要求策略 Send，而不是强行要求策略本身也能无锁共享。
如果接口改为直接共享 Arc<dyn Policy> 并从多个线程调用，通常还需要 Sync，并且必须重新设计可变方法。

| 例子 | 能转移到线程吗 | 能直接跨线程共享引用吗 |
|---|---|---|
| Rc<T> | 否 | 否 |
| RefCell<T>（T: Send） | 可以转移整个值 | 不可以，RefCell 不是 Sync |
| Arc<Mutex<T>> | 取决于 T 等约束 | 用锁协调访问，仍须满足类型约束 |

这些约束由编译器检查，不是执行时才检查。move 只转移捕获值，不会给类型添加 Send。


## `Send` 和 `Sync` 是另一组约束

`dyn Policy` 只说明行为契约，不自动表示它能跨线程使用。需要跨线程时，API 往往明确要求：

```rust
Box<dyn Policy + Send>
Arc<dyn Policy + Send + Sync>
```

粗略理解：

- `Send`：值可以安全地转移到另一个线程；
- `Sync`：多个线程可以安全地共享对该值的引用。

具体实现也必须满足这些约束。`Arc` 的引用计数本身是线程安全的，但 `Arc<T>` 是否能在线程间
共享仍取决于内部 `T` 的能力。

## 自查与关联

为什么 Arc<RefCell<T>> 仍不能直接跨线程共享？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
