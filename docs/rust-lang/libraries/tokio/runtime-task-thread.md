# Runtime、任务、线程与 Mio

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Runtime 驱动任务与 I/O；任务不是专属线程，Mio 只提供底层就绪通知。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：`#[tokio::main]` |
| 阅读时机 | 阶段 5 读到基础自查；Mio/Netty 为可选对照。 |

## 哪些属于 Rust，哪些属于 Tokio

| 概念 | 来自哪里 | 大白话 |
|---|---|---|
| `async`、`.await` | Rust 语言 | 描述异步工作、等待异步结果 |
| `Future`、`pending` | Rust 标准库 | 异步工作协议、永远不完成的 Future |
| Runtime、`spawn`、异步 `JoinHandle` | Tokio crate | 驱动异步任务、启动任务、等待或取消任务 |
| `sleep`、`timeout` | Tokio crate | 异步等待时间、限制等待时间 |
| Router、HTTP 提取器 | Axum crate | 接收 HTTP 请求并调用处理函数 |

Tokio 不会把整个程序自动变成非阻塞程序；放进异步函数的同步阻塞操作，仍然会阻塞线程。
见 [Tokio 官方教程](https://tokio.rs/tokio/tutorial)。

## Runtime、任务和线程

```text
操作系统线程
    └─ Tokio Runtime 调度任务
           ├─ 任务 A：等网络 → 暂停
           └─ 任务 B：有工作 → 继续运行
```

任务是 Runtime 管理的异步工作，不是一任务一线程。单线程 Runtime 也能让多项等待重叠；
多线程 Runtime 还能让任务在不同线程上运行。并发表示多项工作可以交错推进，
并行表示同一时刻真的在执行多项工作。参见 [Tokio spawning 教程](https://tokio.rs/tokio/tutorial/spawning)。

| 宏 | 做什么 | 默认 Runtime |
|---|---|---|
| `#[tokio::main]` | 创建 Runtime 并运行异步入口 | 多线程 |
| `#[tokio::test]` | 为异步测试创建 Runtime | 单线程；每个测试独立 |

默认值不是固定限制，可显式配置。见 [main 宏](https://docs.rs/tokio/1.53.2/tokio/attr.main.html)
和 [test 宏](https://docs.rs/tokio/1.53.2/tokio/attr.test.html)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 单线程 Runtime 中一个任务等网络时，其他已就绪任务有机会推进吗？ |
| 解释 | 为什么任务数量不等于操作系统线程数量？ |
| 边界 | async 函数里的同步阻塞操作能仅靠 async 关键字避免阻塞线程吗？ |

先写下判断和原因，再用自己的已有代码验证。

## 可选：Mio 与 Netty

不需要先掌握 Netty 才能使用 Tokio。

## Mio 与 Netty 放在哪一层

```text
Java                            Rust（按职责粗略类比）
业务 Handler                    async 业务任务
      ↓                               ↓
Netty NioEventLoop               Tokio 调度器 + I/O 驱动
      ↓                               ↓
JDK NIO Selector / Channel       Mio Poll / 网络类型
      ↓                               ↓
操作系统 I/O                    操作系统 I/O
```

Mio 给出非阻塞网络与就绪通知，不提供完整任务调度、线程池和定时器。使用者自己组织事件循环；Tokio 利用它构建网络 I/O 驱动。
Netty NioEventLoop 已组织循环、任务执行和定时任务，所以不应把 Mio 直接等同于 EventLoop。Netty 还有原生 epoll 等实现。

| 需求 | 选择理由 |
|---|---|
| 当前 HTTP 网关、健康检查、超时重试 | Tokio 提供运行时，并能配合当前 Axum/Reqwest 生态 |
| 学习多路复用或自己实现事件循环 | Mio 直接暴露就绪事件，适合作为独立扩展练习 |
| 只想减少开销 | 先测量；使用更底层 API 不保证业务更快 |

Mio 通知可尝试读写，并不保证完整数据已到齐；仍需处理 WouldBlock、部分读写与连接状态。
参考 [Mio](https://github.com/tokio-rs/mio)、[Netty NioEventLoop](https://netty.io/4.1/api/io/netty/channel/nio/NioEventLoop.html)。

## 自查与关联

Tokio、Mio 与 Netty EventLoop 分别在哪一层？

[spawn：启动可独立调度的任务](spawn.md) · [JoinHandle：等任务完成，不是任务本体](join-handle.md) · [sleep 与 timeout：等待和时间上限](time.md)
