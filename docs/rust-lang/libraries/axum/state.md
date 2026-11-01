# State：框架怎样准备共享状态

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**with_state 提供来源，State 的提取规则准备参数，State(state) 模式再拆开包装。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`State(state): State<Arc<GatewayState>>` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 从项目入口到 handler

```text
main 创建 Arc<GatewayState>
        ↓ with_state
Router 保存可提供的状态
        ↓ 请求到达，框架执行提取规则
State<Arc<GatewayState>>
        ↓ State(state) 模式解构
state：Arc<GatewayState>
```

| 名字 | 来源 |
|---|---|
| GatewayState | my-smg 的共享数据结构 |
| State<S> | Axum 的泛型元组结构体 |
| State(state) | Rust 模式，把第 0 个字段绑定给 state |
| State<Arc<GatewayState>> | handler 声明的参数类型 |

对当前同类型状态，提取只需克隆 Arc 句柄，不复制整个 GatewayState。State 还支持通过 FromRef 从较大状态取得子状态，但当前接口没有使用这项扩展。

参考 [Axum 0.8.9 State](https://docs.rs/axum/0.8.9/axum/extract/struct.State.html)。

## 为什么 `AppState` 需要 `Clone`

`Router::with_state(state)` 会把 `state` 移动进 Router。之后 `main` 不能继续使用原变量，除非
传入的是 `state.clone()`：

```text
state ──move──> Router
```

`AppState: Clone` 的直接用途是让 Axum 在构建服务、分发请求时复制状态值或状态句柄。它不表示
每个 handler 都有一份完全独立的底层资源；如果状态内部以后放入 `Arc<T>`，克隆
`AppState` 只会克隆 `Arc`，多个请求仍可指向同一个 `T`。

响应结构体不需要 `Clone`，原因不是它“只活在 handler 内”：handler 会创建响应值，再把所有权
移动给 `Json` 和 HTTP 响应管线。这个过程只有一个所有者，没有复制需求。

```text
HealthResponse ──move──> Json<HealthResponse> ──move──> HTTP 响应管线
```

## 和真实 Axum 怎么对应

| 教学例子 | 真实 Axum |
|---|---|
| `AppState` | 你定义的业务状态 |
| `struct State<T>(T)` | Axum 定义的 `pub struct State<S>(pub S)` |
| main 中的 state | 通过 `Router::with_state` 提供的状态 |
| `FromState` | 本例自定义；Axum 使用 `FromRequestParts` 等 trait |
| `State<T>` 的克隆实现 | Axum 通过 `FromRef` 取得所需状态；同类型的常见实现使用 Clone |
| `dispatch` | 模拟 Handler 调用链中准备参数的部分 |
| 同步 health | 实际通常是异步 handler |

Axum 的 State 提取器还能从较大的状态中取得子状态，并受相应线程安全约束。
完整 HTTP 流程还涉及路由、请求、提取失败、Future 和响应转换，本文没有实现这些机制。

```text
State<AppState>  → 从应用状态准备 AppState
Json<AppState>   → 从请求体 JSON 解析 AppState
```

上面两种写法中，AppState 表示目标数据类型，外面的 State 或 Json 表示不同的提取规则。
本例只实现 State 这一种；同一个 dispatch 也只能使用已有 FromState 实现的目标类型。

## 自查与关联

框架根据参数名字还是参数类型选择提取规则？

[Router 与 Handler：注册和执行是两件事](router-and-handler.md) · [用最小模型理解 Extractor](extractor-mechanics.md) · [Extension：请求内按类型传值](extension.md) · [Json 与 Path：从请求准备参数](json-and-path.md)
