# 原仓库对照指南

[返回学习入口](./index.md) · [上一篇：扩展专题](./03-extensions.md) · [下一篇：学习流程](./05-workflow.md)

先独立实现，再读对应源码，每阶段只读相关文件。下列链接基于 `my-smg` 与 `sgl-model-gateway` 位于同一父目录的布局。

| my-smg 阶段 | 原仓库入口 | 主要看什么 |
|---|---|---|
| 节点与策略 | [worker.rs](../../sgl-model-gateway/src/core/worker.rs)、[policies](../../sgl-model-gateway/src/policies/) | 为什么需要 trait、`Arc` 和策略接口 |
| 同步轮询 | [round_robin.rs](../../sgl-model-gateway/src/policies/round_robin.rs) | 计数器、健康节点筛选、选择结果、边界测试 |
| 配置与错误 | [config](../../sgl-model-gateway/src/config/)、[error.rs](../../sgl-model-gateway/src/core/error.rs) | 校验边界、错误分类、构建方式 |
| HTTP 与流式 | [server.rs](../../sgl-model-gateway/src/server.rs)、[routers](../../sgl-model-gateway/src/routers/) | 请求如何流转、响应如何释放 |
| 并发与节点管理 | [worker_registry.rs](../../sgl-model-gateway/src/core/worker_registry.rs)、[worker_manager.rs](../../sgl-model-gateway/src/core/worker_manager.rs) | 数据由谁拥有、哪些操作需要同步 |
| 可靠性 | [retry.rs](../../sgl-model-gateway/src/core/retry.rs)、[circuit_breaker.rs](../../sgl-model-gateway/src/core/circuit_breaker.rs)、[token_bucket.rs](../../sgl-model-gateway/src/core/token_bucket.rs) | 状态与时间如何影响行为 |
| 队列与工作流 | [job_queue.rs](../../sgl-model-gateway/src/core/job_queue.rs)、[steps](../../sgl-model-gateway/src/core/steps/) | 多步骤任务如何记录和推进 |
| 观测 | [observability](../../sgl-model-gateway/src/observability/) | 在哪些生命周期节点记录数据 |
| 行为验收 | [routing](../../sgl-model-gateway/tests/routing/)、[reliability](../../sgl-model-gateway/tests/reliability/)、[api](../../sgl-model-gateway/tests/api/) | 原项目如何验证边界场景 |
| 依赖边界 | [lib.rs](../../sgl-model-gateway/src/lib.rs)、[Cargo.toml](../../sgl-model-gateway/Cargo.toml) | 哪些能力在当前仓库实现，哪些来自外部 crate |

部分能力在当前仓库中由外部 crate 提供，例如分词、鉴权和 MCP。对应阶段再读取实际依赖源码。

## 每次对照只回答三个问题

1. 原项目比我的实现多处理了什么场景？
2. 这些场景为什么需要额外的类型、共享状态或抽象？
3. 哪些复杂度现在有必要引入，哪些可以留到后续阶段？

记录到本课笔记，模板见[学习流程](./05-workflow.md)。
