# 响应转换：handler 可以返回什么

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**IntoResponse 把字符串、状态码元组或 Json 转成 HTTP 响应。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`chat / health` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 不同返回类型怎样变成响应

| 项目返回值 | HTTP 行为 |
|---|---|
| health 返回 String | 框架将字符串转换成响应正文 |
| chat 返回 (StatusCode, String) | 元组分别确定状态码与正文 |
| remove_worker 返回 StatusCode | 返回状态，正文不携带业务内容 |
| 假节点返回 (StatusCode, Json<ChatResponse>) | 状态加 JSON 序列化正文 |

Axum 通过 IntoResponse 约定执行转换。`(StatusCode, String)` 是当前项目选择的简单接口，不是所有 handler 必须使用的固定类型。

```text
业务处理得到值
    ↓ IntoResponse
HTTP 状态 + 头 + 正文
```

Rust `Result` 的 Err 表示一条失败分支；HTTP 503 是状态码。代码可以返回 `Ok((503, body))`，表示“已取得完整响应，响应的 HTTP 状态为 503”。是否重试由项目策略决定。

返回 Json 与手动返回一个含 JSON 文本的 String 不同：Json 会按库约定序列化并设置相应内容类型。
参考 [Axum IntoResponse](https://docs.rs/axum/0.8.9/axum/response/trait.IntoResponse.html)。

## 阶段 11：SSE 返回的是流式正文

`Sse` 也实现 `IntoResponse`，它持有的流会交给响应正文。handler 返回之后，HTTP 服务继续拉取正文，不需要先生成所有事件。
所有权交接、`poll_frame` / `poll_next`、定时器唤醒及清理时机统一见 [SSE 调度导读](../../walkthroughs/sse-scheduling.md)。
流式输出的缓冲边界与慢客户端处理见 [内存控制与背压](../../advanced/streaming-memory.md)；这些设计材料不表示当前网关已经完成流式转发。

## 自查与关联

返回 Result 和返回 HTTP 500 是同一层概念吗？

[Router 与 Handler：注册和执行是两件事](router-and-handler.md) · [State：框架怎样准备共享状态](state.md) · [用最小模型理解 Extractor](extractor-mechanics.md)
