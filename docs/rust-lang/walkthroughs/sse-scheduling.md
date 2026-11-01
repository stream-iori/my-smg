# SSE 调度：对象、所有权与唤醒流程

[分类目录](index.md) · [总入口](../index.md) · [内存控制与背压](../advanced/streaming-memory.md)

**处理函数构造并交出流，响应正文持有流，HTTP 服务拉取正文，Tokio 在任务可以继续时安排执行。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Future 与 await](../concepts/async/future-and-await.md)、[async move](../concepts/async/async-capture.md)、[Runtime、任务与线程](../libraries/tokio/runtime-task-thread.md)、[响应转换](../libraries/axum/responses.md) |
| 项目入口 | [fake-worker.rs](../../../src/bin/fake-worker.rs)：生成 SSE 的 `chat_stream`；[gateway.rs](../../../src/gateway.rs)：转发字节流的 `chat_stream` |
| 范围 | 阶段 11-01 的假节点调度，以及 11-02 的逐块转发原理。本文不表示网关已经完成流式转发、断开清理或内存上限验收。 |

## 对象清单：谁保存数据，谁推进执行

| 对象 | 保存或负责什么 | 怎样参与执行 |
|---|---|---|
| `Router` / Handler 适配层 | 路由与处理函数的对应关系 | 请求到达后调用 `chat_stream`，驱动它返回响应值 |
| `chat_stream` 的 Future | 提取后的状态与构造响应的过程 | 被框架推进，构造流并返回 `Sse` |
| `Unfold` 流，即 `events` 的实际值 | 生成闭包，以及下一状态或本次生成中的 Future | `poll_next` 推进一次事件生成 |
| 生成闭包 | 接收一个状态元组，返回一个 Future | 每准备生成下一项时调用；本身不是后台任务 |
| 本次生成的 Future | 跨 `.await` 仍需使用的 `part`、`delay`，以及等待进度 | 被流继续推进，完成后交出事件和下一状态 |
| `Sleep` | 本次等待的定时器状态 | 等待时登记唤醒，到期后使相关任务有机会继续 |
| `Sse<流类型>` | 接收并拥有流 | `IntoResponse` 将它转换成 HTTP 响应 |
| `Response<Body>` | 状态、响应头和正文 | 交给 HTTP 服务发送；对象本身不会主动启动任务 |
| `SseBody` | 正文内部的 SSE 包装，拥有流 | `poll_frame` 调用流的 `poll_next`，把事件编码为正文帧 |
| `Event` / `Bytes` / 正文帧 | 当前事件及编码后的字节 | 交给 HTTP 发送流程，不需要积累整个响应 |
| Hyper 连接处理 Future | HTTP 协议状态、正文及读写进度 | 拉取正文、缓冲和写出字节；由 Tokio 管理的任务推进 |
| Tokio 调度器 | 可运行的任务 | 决定什么时候、在哪个工作线程推进任务 |
| Tokio 定时器与 I/O 驱动 | 定时器、网络就绪通知 | 条件满足时，通过任务的 `Waker` 发出唤醒通知 |
| 操作系统 Socket / TCP 缓冲 | 尚未写出或尚未被对端接收的数据 | 网络传输；容量和进度会影响服务端继续发送 |
| 客户端，例如 `curl` | 接收并消费字节 | 读取速度影响背压；`curl -N` 关闭 curl 的输出缓冲 |

表中的包装不要求一层对应一个堆分配；多个 Future 可以嵌套在同一个任务中。这里的 `unfold`、生成 Future、`Sleep` 都没有各自调用 `spawn`。

## 图一：所有权怎样交接

```text
构造阶段（箭头表示 move，不表示执行频率）

chat_stream 中的局部变量 events
    |
    | Sse::new(events)，按值接收
    v
Sse { stream: 流 }
    |
    | 返回给框架，框架调用 IntoResponse
    v
HTTP Response
    +-- 状态与响应头
    +-- Body
         +-- SseBody
              +-- 流 Unfold
                   +-- 生成闭包
                   +-- 下一状态 (part, delay)
                       或本次生成中的 Future
```

这些交接发生在构造和转换时。进入发送阶段后，HTTP 服务通过借用推进正文和流，不需要把流交回处理函数。

`events` 移入 `Sse` 后，原局部变量不能再使用；处理函数退出也不会通过这个已移动的变量销毁流。
`state.delay` 是 `Duration`，这里复制了一份交给流，流没有借用处理函数的局部 `state`。

## 图二：调用链与唤醒链

```text
调用、推进：

Tokio 调度连接任务
    |
    v
Hyper 推进 HTTP 发送
    |
    v
Body.poll_frame(cx)
    |
    v
SseBody --> Stream.poll_next(cx)
                 |
                 v
            本次生成 Future.poll(cx)
                 |
                 v
            Sleep.poll(cx)（需要等待时）

等待未就绪：Pending 沿调用链返回，任务让出执行机会

唤醒、恢复：

定时器到期 / Socket 恢复可写
    |
    v
通过 Waker 通知任务可以再次尝试
    |
    v
Tokio 调度任务 --> 从上述调用链继续推进
```

`cx` 是本次推进使用的上下文，其中有任务的 `Waker`。返回 `Pending` 的异步组件需要正确登记后续通知；这里只使用库已经实现好的 `Sleep` 和网络组件。
唤醒不等于立即执行，也不保证这一次就能完成；执行器稍后再次推进，组件仍可能返回 `Pending`。
这不是按固定周期扫描所有流，也不需要为每条 SSE 事件创建线程。

## 图三：三条事件怎样逐段交出

假设段间隔为 500ms，客户端持续读取，忽略调度和网络延迟：

```text
时间       HTTP 服务 / 流                         客户端
约 0ms     拉取 --> part=1 --> 交出事件 ----------> part-1
           再拉取 --> part=2 --> Sleep --> Pending
                                 |
约 500ms                   定时器唤醒
           继续推进 --> 交出事件 ----------------> part-2
           再拉取 --> part=3 --> Sleep --> Pending
                                 |
约 1000ms                  定时器唤醒
           继续推进 --> 交出事件 ----------------> part-3
           再拉取 --> part=4 --> None --> 正文结束
```

| `poll_next` 的结果 | 含义 | 之后的动作 |
|---|---|---|
| `Pending` | 下一项暂时没准备好 | 暂停并等待通知 |
| `Ready(Some(Ok(event)))` | 得到一条事件 | 编码后交给发送流程，随后可继续拉取 |
| `Ready(Some(Err(error)))` | 生成流的一项时失败 | 由正文和连接层处理错误；本例使用 `Infallible`，没有这个生成分支 |
| `Ready(None)` | 没有下一项 | 结束正文，之后随相关对象清理 |

不用等三条全部生成。若多项已经就绪，HTTP 服务也可以连续拉取并适当缓冲；每次交出事件不等于客户端已收到或确认消费。
SSE 事件、HTTP 正文帧与 TCP 包是不同边界，不能认为一条事件必然对应一个 TCP 包。
本例的 `Sleep` 在该项第一次被推进时开始，慢客户端可能使下一段更晚开始生成，因此不是严格的固定频率推送。

## 阶段 11-02：不收集完整响应是什么意思

**得到一块正文，就可以把这一块交给发送流程，无须等到上游整个响应结束。**

网关新增转发处理函数中的两行：

```rust
let stream = upstream.bytes_stream();
let body = Body::from_stream(stream);
```

| 表达式 | 类型或作用 | 是否在这里读完整响应 |
|---|---|---|
| `upstream.bytes_stream()` | 消费上游响应，得到逐块产生 `Result<Bytes, reqwest::Error>` 的流 | 否，只交接读取正文的能力 |
| `Body::from_stream(stream)` | 让下游正文持有这个字节流 | 否，只包装流 |
| HTTP 服务后续推进正文 | 拉取下一块，交给发送流程 | 按发送进度逐步读取，不要求先收集完整正文 |
| `upstream.text().await` | 收集完整正文，再得到一个 `String` | 是，需要等正文读完 |

```text
完整读取路径：

上游     第 1 块 ----> 第 2 块 ----> 第 3 块 ----> 结束
网关       收集         收集         收集          |
                                                 v
                                           得到完整 String
                                                 |
                                                 v
                                            交给客户端

逐块转发路径：

上游     第 1 块 ----> 第 2 块 ----> 第 3 块 ----> 结束
            |            |            |
网关       转发         转发         转发
            |            |            |
客户端     可收到       可收到       可收到
```

“有多少就先吐多少”可以作为直觉，但要保留以下边界：

| 边界 | 准确理解 |
|---|---|
| 字节块不是单个字节 | HTTP 库决定本次交出的字节块，网络和协议层仍有缓冲 |
| 字节块不是 SSE 事件 | 一块可能只有半条事件，也可能包含多条；透明转发无须先解析出完整事件 |
| 交出不等于对端已收到 | HTTP 服务还需要编码和写出，客户端也可能有缓冲 |
| 数据就绪不等于无限读取 | 下游写不动时，缓冲限制会反向影响继续拉取，见[内存控制与背压](../advanced/streaming-memory.md) |

假节点已经用 `Event` 编码了 SSE 字节，网关只需透明转发；再次调用 `Event::data` 会把已有协议文本当成新数据，再编码一遍。
本例不把 `Stream` 收集到 `Vec` 或完整 `String`，也没有额外创建后台生产队列。

```text
下游 Body 持有字节流
          |
HTTP 服务拉取下一帧
          |
          v
字节流推进上游正文读取
          |
          +-- 有字节块 --> 交给下游发送
          +-- Pending --> 等上游网络就绪后再推进
          +-- 结束    --> 下游正文随之结束
```

因此，handler 返回后，上游正文的读取能力由字节流继续持有，而字节流由下游正文持有。
需要保留的响应头要在 `bytes_stream()` 消费上游响应之前保存；为何这里使用 `.cloned()`，见 [clone 与 cloned 的阶段 11 应用](../concepts/ownership/copy-and-clone.md#阶段-11先保存响应头再移动上游响应)。

## 流内部也有状态交接

闭包参数是 `|(part, delay)|`：接收一个元组，并将它拆开。它不同于接收两个参数的 `|part, delay|`。

```text
流保存 (2, delay)
    |
    | 调用闭包，状态交给本次 Future
    v
Future 持有 part、delay，等待 Sleep
    |
    | 等待结束后继续
    v
Some((Ok(本次事件), (3, delay)))
          |              |
          v              v
       交出事件      流保存下一状态
```

Future 暂停时没有被销毁。编译器生成的状态保存跨 `.await` 仍要使用的数据；正常恢复时可以继续使用。
若 Future 被销毁，保存的数据也随之清理，等待后的代码不会继续运行。
`1_usize` 与 `1usize` 等价，表示初始编号 1 的类型明确为 `usize`。

## SSE 的空行结束的是一条事件

```text
data: A\n\n         --> 第一条事件，数据为 A
data: B\n\n         --> 第二条事件，数据为 B

data: A\ndata: B\n\n --> 一条事件，数据为 A\nB
```

这里用 `\n` 表示换行。空行分隔事件，不会关闭连接；整个 HTTP 正文结束才表示这个流结束。
Axum 的 `Event` 编码会处理字段前缀和事件结束标记，调用 `.data(...)` 时只需要提供数据。

## 结束、取消与资源生命周期

| 情况 | 本地发生什么 | 需要保留的边界 |
|---|---|---|
| 正常 `None` | 正文报告结束，框架随后清理正文及其字段 | 报告结束与对象 `Drop` 不是必须发生在同一行的动作 |
| 网络断开被检测到 | HTTP 服务结束相关发送并清理正文 | 物理断网到被检测到可能有延迟，需要期限策略 |
| 正文或所属任务被销毁 | 流及正在等待的生成 Future 随拥有者销毁 | 本例没有独立生产任务需要额外停止 |
| 以后加入 `spawn` 生产任务 | 生命周期形成了独立分支 | 丢弃 `JoinHandle` 不会自动取消它，需要明确取消和回收责任 |

阶段 11 后续设计执行凭证和在途计数时，资源区间要覆盖正文发送。只把 guard 留在 handler 局部，会在 handler 返回时提前释放。
正文要求的数据必须能独立存活；借用 handler 局部变量的 guard 不能直接随正文返回，要重新设计其拥有关系。
本地停止读取上游，也不证明远端已经撤销业务计算。

## 源码依据与核对方法

本文按当前项目代码及本机已解析的依赖源码核对，不依赖上游网关的实现。
库内包装和连接调度属于版本相关细节，升级后可重新搜索下表符号。

| 版本 | 依赖源码路径（相对 crate 根） | 核对符号与结论 |
|---|---|---|
| Axum 0.8.9 | `src/response/sse.rs` | `Sse::new` 保存流；`IntoResponse for Sse` 将流移入正文；`SseBody::poll_frame` 拉取一项并编码 |
| axum-core 0.5.6 | `src/body.rs` | `Body::from_stream` 保存传入的流，后续推进正文时才拉取数据 |
| Reqwest 0.13.5 | `src/async_impl/response.rs` | `Response::bytes_stream` 消费响应并包装正文流；需要启用 `stream` feature |
| futures-util 0.3.34 | `src/stream/unfold.rs` | `unfold`、`Unfold::poll_next` 保存状态或本次 Future，并交出下一项 |
| Hyper 1.11.1 | `src/proto/h1/dispatch.rs` | `poll_write` 驱动 HTTP/1 正文拉取与写出 |
| Tokio 1.53.1 | `src/sync/mpsc/bounded.rs`、`src/runtime/task/join.rs` | 有界队列的等待行为；丢弃任务句柄不会自动取消任务 |

依赖源码位于 Cargo 缓存的 `registry/src/<registry>/<crate>-<version>/`。版本以本地 `Cargo.lock` 为准；这里描述的 HTTP/1 发送细节不直接替代 HTTP/2 的实现分析。

## 自查与关联

| 问题 | 应能说明的点 |
|---|---|
| handler 返回后谁拥有流？ | 响应正文；局部变量移走后不再负责销毁它 |
| 谁保证睡眠结束后有机会继续？ | 定时器通过 Waker 通知，Tokio 调度，HTTP 服务再次推进 |
| 三条一起发吗？ | 不需要等全部生成，但底层可能缓冲或合并字节 |
| `bytes_stream` 与 `Body::from_stream` 会读完整正文吗？ | 构造时交接和包装读取能力，后续由 HTTP 服务逐块推进 |
| 慢客户端会怎样？ | 写出进度通过缓冲限制反向影响正文拉取，具体边界见内存文章 |

[流式响应的内存控制与背压](../advanced/streaming-memory.md) · [Drop 与 RAII](../concepts/ownership/drop-and-raii.md) · [任务清理](task-cleanup.md)
