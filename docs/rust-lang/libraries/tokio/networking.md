# 异步网络：bind、accept 与分段读写

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**监听器接收连接，流读写字节；一次 read 不保证拿到完整 HTTP 请求。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Runtime、任务、线程与 Mio](runtime-task-thread.md) · [共享借用与可变借用](../../concepts/ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`start_slow_body_server` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 沿测试服务器读一次网络调用

`src/gateway/tests/support.rs::start_slow_body_server` 使用监听器接收一个连接，再累积字节直到请求头完整。

```text
TcpListener::bind("127.0.0.1:0")
    ↓ OS 分配本地随机端口
listener.local_addr() → 测试取得实际地址
    ↓
listener.accept().await → 一个 TcpStream
    ↓
read().await → 可能只读到部分字节
    ↓
write_all().await → 发送全部给定字节，或返回错误
```

| API | 结果与边界 |
|---|---|
| bind | 创建监听器；这里端口 0 让并行测试避免固定端口冲突 |
| accept | 得到连接流和对端地址，不是 HTTP 请求对象 |
| read | 返回本次读到的字节数；0 通常表示流结束 |
| write_all | 在函数内部处理部分写入，失败返回错误 |

测试用 `received.extend_from_slice(&buffer[..count])` 累积本次有效部分，不能把整个未填满缓冲区都追加。
检测 `b"\r\n\r\n"` 只是这个测试识别请求头边界的办法，不是完整 HTTP 解析器。

项目故意先发 200 响应头，等待后再发正文，从而单独观察 `text().await` 超时；收到响应头不等于整个请求成功。

这些是现有测试代码的导读，不是要求新写服务器。参考 [Tokio TcpListener](https://docs.rs/tokio/1.53.2/tokio/net/struct.TcpListener.html)。

## 自查与关联

为什么要累积数据直到读到请求头结束标记？

[Runtime、任务、线程与 Mio](runtime-task-thread.md) · [spawn：启动可独立调度的任务](spawn.md) · [JoinHandle：等任务完成，不是任务本体](join-handle.md)
