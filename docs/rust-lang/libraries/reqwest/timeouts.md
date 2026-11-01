# 单次请求超时与总超时

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**请求 timeout 限制一次请求过程，外层 Tokio timeout 限制整个重试流程。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`.timeout(Duration::from_secs(1))` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 两个时间边界

| 项目配置 | 范围 |
|---|---|
| RequestBuilder 的 1 秒 timeout | 本次请求从连接到正文读取结束的整个过程 |
| Tokio 的 2 秒 timeout | 完整重试 Future，包括请求、正文读取和退避等待 |

```text
总期限 2 秒
├── 第一次请求（每次最多 1 秒）
├── 退避 sleep
├── 第二次请求（每次最多 1 秒）
└── 到总期限时，不再继续驱动完整重试 Future
```

这不承诺一定执行满三次：次数上限与总时间上限谁先到，谁先限制流程。
超时只表示本地停止等待，不能证明上游没有收到请求或没有产生副作用。未来流式响应开始发送后，不能直接套用完整响应重试方案。

当前测试 `start_slow_body_server` 故意延迟正文，用来覆盖已收到状态码但正文尚未完成的超时路径。
参考 [RequestBuilder::timeout](https://docs.rs/reqwest/0.13.5/reqwest/struct.RequestBuilder.html#method.timeout)。

## 自查与关联

1 秒与 2 秒这两层超时各覆盖什么？

[Client 与请求构建](client-and-request.md) · [响应头、正文与错误分类](response-and-errors.md)
