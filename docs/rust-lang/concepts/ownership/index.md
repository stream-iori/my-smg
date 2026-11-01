# 所有权、借用与释放

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Move 与部分移动](move-and-partial-move.md) | 为什么未移动的字段有时还能用，而整个对象不能用？ | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md) | [src/worker.rs](../../../../src/worker.rs)：`Worker::new` |
| [共享借用与可变借用](borrowing.md) | 为什么 policy 是 &mut，而 workers 是共享切片？ | [Move 与部分移动](move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：`Policy::select` |
| [self、mut self、&self 与 &mut self](method-receivers.md) | 接收者简写、自动借用与 `self: &Arc<Self>` 各是什么意思？ | [共享借用与可变借用](borrowing.md)基础部分；Arc 接收者在阶段 11 阅读。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`ManagedTask::cancel_and_join`；[src/worker.rs](../../../../src/worker.rs)：`begin_request_owned` |
| [Copy 与 Clone：复制值还是共享句柄](copy-and-clone.md) | Arc/String 的 clone 成本相同吗？Option 的 clone 与 cloned 分别得到什么？ | [Move 与部分移动](move-and-partial-move.md)基础部分；Option 与响应头应用按需阅读。 | [src/circuit_breaker.rs](../../../../src/circuit_breaker.rs)：`CircuitState`；[src/gateway.rs](../../../../src/gateway.rs)：`chat_stream` |
| [生命周期：引用不能比数据活得久](lifetimes.md) | 借用型 guard 的 'a、Drop 实现与拥有型 guard 怎样区分？ | [共享借用与可变借用](borrowing.md)；Arc 对照在阶段 11 阅读。 | [src/worker.rs](../../../../src/worker.rs)：`InFlightGuard<'a> / OwnedInFlightGuard` |
| [Drop 与 RAII：离开作用域如何收尾](drop-and-raii.md) | 为什么提前 return 也会让在途计数减一？ | [Move 与部分移动](move-and-partial-move.md) | [src/worker.rs](../../../../src/worker.rs)：`impl Drop for InFlightGuard` |
