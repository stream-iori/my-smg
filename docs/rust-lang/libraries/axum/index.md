# Axum

[总入口](../../index.md) · [问题索引](../../questions.md)

以当前 Cargo.lock 锁定的 **0.8.9** 为例。Manifest 的版本范围和最终锁定版本要分开看。

[对应版本官方文档](https://docs.rs/axum/0.8.9/axum/)。按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Router 与 Handler：注册和执行是两件事](router-and-handler.md) | 注册路由时 chat 已执行了吗？ | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/main.rs](../../../../src/main.rs)：`.route("/chat", post(chat))` |
| [State：框架怎样准备共享状态](state.md) | 框架根据参数名字还是参数类型选择提取规则？ | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`State(state): State<Arc<GatewayState>>` |
| [Extension：请求内按类型传值](extension.md) | 中间件怎样把请求 ID 交给处理函数？ | [TypeId 与 Any](../../concepts/types/type-id-and-any.md) · [用最小模型理解 Extractor](extractor-mechanics.md) | [src/main.rs](../../../../src/main.rs)：`request_id_middleware`；[src/gateway.rs](../../../../src/gateway.rs)：`chat` |
| [用最小模型理解 Extractor](extractor-mechanics.md) | 只定义 State 结构体是否就具备提取能力？ | [泛型、类型推断与 turbofish](../../concepts/types/generics-and-inference.md) · [Trait：类型需要提供什么能力](../../concepts/types/traits-and-bounds.md) · [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) | [src/gateway.rs](../../../../src/gateway.rs)：`chat` |
| [Json 与 Path：从请求准备参数](json-and-path.md) | State、Json、Path 分别从哪里取数据？ | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`add_worker / remove_worker` |
| [响应转换：handler 可以返回什么](responses.md) | 返回 Result 和返回 HTTP 500 是同一层概念吗？ | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) | [src/gateway.rs](../../../../src/gateway.rs)：`chat / health` |
