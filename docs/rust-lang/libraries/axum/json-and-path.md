# Json 与 Path：从请求准备参数

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Json 消费并解析请求体，Path 从路由参数取值；类型明确目标数据。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`add_worker / remove_worker` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 参数类型决定取值来源

项目的两个处理器签名节选：

```rust
pub(super) async fn add_worker(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<WorkerConfig>,
) -> (StatusCode, String)
```

```rust
pub(super) async fn remove_worker(
    State(state): State<Arc<GatewayState>>,
    AxumPath(worker_id): AxumPath<String>,
) -> StatusCode
```

以上只展示签名，省略函数体；AxumPath 是对 `axum::extract::Path` 的导入别名。

| 参数 | 从哪里准备 | handler 拿到什么 |
|---|---|---|
| State | Router 的应用状态 | Arc<GatewayState> |
| Json | HTTP 请求体 | 反序列化后的 WorkerConfig |
| AxumPath | `/workers/{worker_id}` 路由参数 | String |

```text
POST /workers + JSON → Json<WorkerConfig> → request
DELETE /workers/a    → Path<String>       → worker_id = "a"
```

提取成功才调用 handler。JSON 格式或类型错误属于提取失败；结构正确但地址不合法，则由项目的配置校验返回 400。它们是不同层。
Json 需要消费请求体，因此放在这类 handler 的最后一个参数位置。State/Path 等不消费正文的提取器可以先执行。

参考 [Axum Json](https://docs.rs/axum/0.8.9/axum/struct.Json.html)、[Path](https://docs.rs/axum/0.8.9/axum/extract/struct.Path.html)。

## 自查与关联

State、Json、Path 分别从哪里取数据？

[Router 与 Handler：注册和执行是两件事](router-and-handler.md) · [State：框架怎样准备共享状态](state.md) · [用最小模型理解 Extractor](extractor-mechanics.md)
