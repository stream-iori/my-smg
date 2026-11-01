# Reqwest

[总入口](../../index.md) · [问题索引](../../questions.md)

以当前 Cargo.lock 锁定的 **0.13.5** 为例。Manifest 的版本范围和最终锁定版本要分开看。

[对应版本官方文档](https://docs.rs/reqwest/0.13.5/reqwest/)。按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Client 与请求构建](client-and-request.md) | Axum post 和 Reqwest post 分别做什么？ | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`state.client / client.post(url_ref)` |
| [响应头、正文与错误分类](response-and-errors.md) | 收到 503 为什么仍可能是 Ok(response)？ | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`upstream.status() / upstream.text().await` |
| [单次请求超时与总超时](timeouts.md) | 1 秒与 2 秒这两层超时各覆盖什么？ | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`.timeout(Duration::from_secs(1))` |
