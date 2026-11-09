# Pin 与 Unpin：固定的是谁

[分类目录](index.md) · [Poll、Context 与 Waker](poll-context-waker.md) · [问题索引](../../questions.md)

**`Pin` 约束指针指向的值；`Unpin` 是标准库的 trait，表示该类型不需要固定内存地址。** 下文直接区分“实现 `Unpin`”和“未实现 `Unpin`”两种情况。

| 阅读准备 | 内容                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------- |
| 前置     | [Poll、Context 与 Waker](poll-context-waker.md) · [所有权与移动](../ownership/move-and-partial-move.md) |
| 项目入口 | [写停滞超时](../../../../src/write_timeout_listener.rs)：`write_timer`、`check_timeout`、`poll_write`   |

## 为什么需要 Pin

一些值在运行过程中可能对自身地址有要求。例如编译器生成的 Future 可能需要保存跨等待点的状态，某些状态可能涉及内部引用。若底层实现依赖地址稳定，固定后再移动这个值可能使这些关系失效。

`Future::poll` 因此统一接收 `Pin<&mut Self>`：调用者要提供被固定的可变访问，让需要地址稳定的 Future 能安全推进。并非每个 Future 都有自引用；也有实现了 `Unpin` 的 Future。

```text
Pin<指针> ──约束──▶ 指针指向的值 T
                     └─ T 未实现 Unpin：固定后必须保持原位，直到原地销毁
```

固定约束从固定开始持续到值被原地销毁；
允许正常 `drop`，并不是永远不释放。
对于实现了 `Unpin` 的类型，安全代码可以解除固定，再移动里面的值。
对于未实现 `Unpin` 的类型，值在**尚未固定时**可以正常移动；建立固定保证后，才必须保持原位，直到原地销毁。

## 实现 Unpin 与未实现 Unpin

| 状态        | 含义                                       | 例子                   |
| ----------- | ------------------------------------------ | ---------------------- |
| T 实现 `Unpin`（代码写作 `T: Unpin`） | 可以安全地解除固定，再移动 T | 项目使用的 `TcpStream` |
| T 未实现 `Unpin` | 固定后必须保持原位，直到原地销毁 | Tokio `Sleep` |

`Unpin` 是自动 trait，多数类型自动实现；组合类型是否实现取决于其字段和实现方式。它描述类型的性质，本身不是解除固定的操作。

## Pin::new 与 Box::pin

|              | `Pin::new(&mut value)`             | `Box::pin(value)`              |
| ------------ | ---------------------------------- | ------------------------------ |
| 输入         | 现有值的可变引用                   | 值本身                         |
| 输出         | `Pin<&mut T>`：借用                | `Pin<Box<T>>`：拥有所有权      |
| 分配         | 不分配                             | 通常分配堆内存，零大小类型例外 |
| 安全调用条件 | 对这里的 `&mut T`，要求 `T: Unpin` | 也可用于未实现 `Unpin` 的类型 |

安全的 `Pin::new` 是对已有指针包装；`Box::pin` 相当于创建 `Box` 后将其转成固定指针。如果已有 `Box<T>`，可以使用 `Box::into_pin`，无需再次分配。未实现 `Unpin` 的局部值也可使用标准库 `pin!` 固定，不一定要堆分配。

## 项目中的两种用法

```text
self.inner: TcpStream
  └─ Pin::new(&mut self.inner) → Pin<&mut TcpStream> → poll_write(cx)

新建 Sleep
  └─ Box::pin(...) → Pin<Box<Sleep>> → 存入 self.write_timer
                                          └─ as_mut() → Pin<&mut Sleep> → poll(cx)
```

`TcpStream` 可以用 `Pin::new` 创建临时固定借用，以满足 `poll_write` 的接口。`Sleep` 未实现 `Unpin`，这里用 `Box::pin` 将其固定并持有，方便保存到结构体里跨多次 poll 使用。

移动 `Pin<Box<Sleep>>` 指针本身，不会移动堆里的 `Sleep`。当 `write_timer` 被设置为 `None` 时，原来的 `Sleep` 可以正常原地销毁。不要把“固定值的地址”理解为“不能移动指针”或“不能释放值”。

## 自查

1. 一个未实现 `Unpin` 的值在固定之前能否移动？固定后能否销毁？
2. 为什么移动 `Pin<Box<Sleep>>` 不会移动堆里的 `Sleep`？
3. 项目为什么对 `TcpStream` 使用 `Pin::new`，对 `Sleep` 使用 `Box::pin`？

参考：[标准库 Pin](https://doc.rust-lang.org/std/pin/) · [Pin::new](https://doc.rust-lang.org/std/pin/struct.Pin.html#method.new) · [Box::pin](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.pin) · [Tokio Sleep](https://docs.rs/tokio/latest/tokio/time/struct.Sleep.html)。
