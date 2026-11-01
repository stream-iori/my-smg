# Client 与请求构建

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**复用 Client 复用连接池，post/json 等构建请求，send 返回异步请求过程。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`state.client / client.post(url_ref)` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 项目只创建可复用的 Client

`main` 构造 Client 并放进 GatewayState；每次 chat 借用 `&state.client`，而不是重新建一个客户端。
Client 内部管理连接池；克隆 Client 也共享内部资源，不应按“复制所有连接”理解。

项目请求链的结构摘录：

```rust
client
    .post(url_ref)
    .json(request_ref)
    .timeout(Duration::from_secs(1))
    .send()
    .await
```

| 步骤 | 负责什么 |
|---|---|
| post | 选择 HTTP 方法和地址，得到 RequestBuilder |
| json | 序列化请求数据并设置请求正文 |
| timeout | 设置这次请求的时间上限 |
| send | 创建并执行异步发送过程；完成时得到 Response 或 Error |

```text
构建请求 → send().await → Response → text().await → String
```

泛型接口支持的输入取决于具体 trait 实现，不是“引用都能自动转成任意类型”。
参考 [Reqwest 0.13.5 Client](https://docs.rs/reqwest/0.13.5/reqwest/struct.Client.html)。

## 为什么 `client.post(...)` 两种写法都可以？

当前项目 Reqwest 0.13.5 的参数约束是 `U: IntoUrl`。它分别为 `&str` 和 `&String`
实现了 IntoUrl，所以 `client.post(url.as_str())` 和 `client.post(&url)` 都可以。
这不意味着两者类型相同，也不能简单归因于自动解引用。

作为对照，普通函数明确要求 `&str` 时，`&String` 可以发生解引用强制转换：

```rust
fn inspect(text: &str) -> usize {
    text.len()
}

inspect(url.as_str()); // 已经是 &str
inspect(&url);        // 转成函数需要的 &str
```

重试循环中借用 url，每轮都能再次使用它；直接按值传入 String 则会转移所有权。
“不复制”只描述借用表达式本身，Reqwest 后续解析 URL、构造请求仍可能分配内存。

依据：[Reqwest IntoUrl 实现](https://docs.rs/reqwest/0.13.5/src/reqwest/into_url.rs.html)、
[String::as_str](https://doc.rust-lang.org/std/string/struct.String.html#method.as_str)。
关联：[Axum post 与 Reqwest post](client-and-request.md)。

## 两个 post 不要混淆

| 调用 | 所属库 | 返回什么 | 做什么 |
|---|---|---|---|
| `post(handler)` | Axum | MethodRouter | 注册接收 POST 请求的规则 |
| `client.post(url)` | Reqwest | RequestBuilder | 准备向上游发送 POST 请求 |
| `builder.send()` | Reqwest | Future | 驱动后发送请求，完成结果为 Result |

```text
客户端 → Axum 网关接收请求 → Reqwest client 发请求 → 上游 Worker
```

URL 参数区别见 [url.as_str() 与 &url](../../concepts/values/string-and-str.md)。

## 自查与关联

Axum post 和 Reqwest post 分别做什么？

[响应头、正文与错误分类](response-and-errors.md) · [单次请求超时与总超时](timeouts.md)
