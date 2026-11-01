# 项目代码导读

[总入口](../index.md) · [问题索引](../questions.md)

按[当前阶段](../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [SSE 调度：对象、所有权与唤醒流程](sse-scheduling.md) | handler 返回后谁持有流，谁拉取事件，谁负责唤醒？ | [Future](../concepts/async/future-and-await.md)、[async move](../concepts/async/async-capture.md)、[Runtime](../libraries/tokio/runtime-task-thread.md) | [src/bin/fake-worker.rs](../../../src/bin/fake-worker.rs)：`chat_stream`；阶段 11 阅读 |
| [代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) | 请求被取消时，谁负责计数减一？ | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Drop 与 RAII：离开作用域如何收尾](../concepts/ownership/drop-and-raii.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 | [src/worker.rs](../../../src/worker.rs)：`Worker / InFlightGuard` |
| [代码导读：轮询选择与策略状态](policy-selection.md) | 每次新建 RoundRobin 为什么总从第一个开始？ | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../src/policy.rs)：`RoundRobin::select` |
| [代码导读：配置加载与分层错误](configuration-loading.md) | 哪一层决定错误怎样打印，哪一层保留底层原因？ | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) · [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) · [JSON 解析与业务校验](../libraries/serde/json-parsing-and-validation.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../src/config.rs)：`GatewayConfig::from_file` |
| [代码导读：一次聊天请求](chat-forwarding.md) | 为什么取得快照后才能释放注册表锁并等待网络？ | [State：框架怎样准备共享状态](../libraries/axum/state.md) · [Client 与请求构建](../libraries/reqwest/client-and-request.md) · [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | [src/gateway.rs](../../../src/gateway.rs)：`chat` |
| [代码导读：重试与总期限](retry-and-timeout.md) | Err(response) 与上游 503 是否走相同的重试分支？ | [关联类型：实现确定自己的 Output](../concepts/types/associated-types.md) · [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) · [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | [src/gateway/retry.rs](../../../src/gateway/retry.rs)：`retry_with_backoff` |
| [代码导读：ManagedTask 的清理责任](task-cleanup.md) | Drop 的兜底和显式 cancel_and_join 分别保证什么？ | [self、mut self、&self 与 &mut self](../concepts/ownership/method-receivers.md) · [Option 中的借用与取值：as_ref、as_deref、take](../concepts/errors/option-access-and-take.md) · [取消：abort 之后为什么还要 await](../libraries/tokio/cancellation.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../src/gateway/tests/support.rs)：`ManagedTask / TestServer` |
| [代码导读：许可、状态版本与迟到结果](circuit-permit.md) | 为什么 Arc<()> 身份比只比较状态枚举更有用？ | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) · [Drop 与 RAII：离开作用域如何收尾](../concepts/ownership/drop-and-raii.md)基础部分；项目应用另按节标题补前置。 | [src/circuit_breaker/gate.rs](../../../src/circuit_breaker/gate.rs)：`CircuitPermit` |
