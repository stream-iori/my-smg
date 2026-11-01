# Extension：请求内按类型传值

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**`Extension<T>` 是提取器；数据保存在请求的 `http::Extensions` 类型映射里。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [TypeId 与 Any](../../concepts/types/type-id-and-any.md) · [用最小模型理解 Extractor](extractor-mechanics.md) · [State](state.md) |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：`request_id_middleware`；[src/gateway.rs](../../../../src/gateway.rs)：`chat` |
| 阅读时机 | 阶段 12；以下源码说明对应 Axum 0.8.9、http 1.5.0。 |

## 项目中的数据流

```text
请求到达
   ↓
中间件生成 RequestId，并插入 request.extensions_mut()
   ↓ next.run(request)
Axum 从请求部件提取 Option<Extension<RequestId>>
   ↓
chat 取出 ID，记录选中的节点
   ↓
中间件把同一个 ID 显式写入响应头 x-request-id
```

请求 A 和请求 B 各有自己的 extensions 集合；并发请求不会共享“当前请求 ID”变量。响应头与请求扩展是两份不同的数据：扩展不会自动发给客户端或上游。发送上游时也必须显式构造请求头。

| 项目写法 | 作用 |
|---|---|
| `request.extensions_mut().insert(RequestId(id))` | 在当前请求中保存值 |
| `next.run(request)` | 把请求交给后续处理链 |
| `Option<Extension<RequestId>>` | 按类型提取，允许缺失 |
| `Some(Extension(RequestId(id)))` | 用模式逐层解构，得到数字 |

## 内部实现：插入、查找、提取

`http::Extensions` 在未使用时不创建 Map；第一次插入时创建。它按 `TypeId` 存储类型擦除后的值，同一类型再次插入会替换旧值。取值时以目标类型的 `TypeId` 查找，再通过 `Any` 检查并向下转换。具体逻辑见 [http 1.5.0 Extensions 源码](https://docs.rs/http/1.5.0/src/http/extensions.rs.html)。

```text
插入：RequestId(2) → TypeId<RequestId> → Map 中的类型擦除值
读取：RequestId    → TypeId<RequestId> → 查 Map → 检查并取得 &RequestId
```

Axum 的 `Extension<T>` 从请求部件的 extensions 读取 `T`，然后**克隆 `T`**并包装成 `Extension<T>`；它不是把 Map 中的 `&T` 直接交给处理函数。`RequestId` 是小值；如果 `T` 是 `Arc<LargeObject>`，克隆的是共享句柄而非整个对象。源码见 [Axum 0.8.9 Extension](https://docs.rs/axum/0.8.9/src/axum/extension.rs.html#71-112)。

| 约束 | 用途 |
|---|---|
| `Clone` | 提取器克隆保存的值 |
| `Send + Sync` | 满足跨线程传递和共享引用的安全约束 |
| `'static` | 类型不依赖短生命周期借用；不表示值永不释放 |

## 必需和可选

| 处理函数参数 | 有值 | 缺失 |
|---|---|---|
| `Extension<RequestId>` | 提取并继续 | Axum 拒绝请求，默认返回 500 |
| `Option<Extension<RequestId>>` | 得到 `Some` | 得到 `None`，继续处理 |

当前 `chat` 选择可选：直接调用的测试可以传 `None`；代价是遗漏中间件时，节点日志不会带请求 ID。当前 `chat_stream` 尚未提取该 ID。

## 与 State、Json 的区别

| 提取器 | 在当前项目中的来源 | 作用 |
|---|---|---|
| `Extension<RequestId>` | 当前请求的 extensions | 请求级信息 |
| `State<Arc<GatewayState>>` | Router 的应用状态 | 跨请求共享节点、策略等 |
| `Json<ChatRequest>` | HTTP 请求正文 | 解析客户端提交的数据 |

`Extension` 也可用于提供共享数据；上表是当前项目的选择。`Extension` 从请求部件提取而不消费正文；`Json` 消费正文，因此在 handler 参数中需要放在其他部件提取器之后。

## 自查

1. 为什么请求扩展中的 ID 不会自动出现在 HTTP 响应头里？
2. 如果中间件没有插入 ID，可选提取和必需提取分别如何处理？
3. 如果扩展保存 `Arc<GatewayState>`，提取时会复制整个 `GatewayState` 吗？
