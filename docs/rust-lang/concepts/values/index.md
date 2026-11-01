# 类型、值与容器

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [struct、enum、impl 与 Self](struct-enum-and-impl.md) | Worker::new 与 worker.id 分别如何调用？ | 无；可以直接阅读。 | [src/worker.rs](../../../../src/worker.rs)：`Worker / HealthStatus / impl Worker` |
| [元组、元组结构体与 unit](tuples-and-unit.md) | () 与 (StatusCode, String) 携带多少信息？ | 无；可以直接阅读。 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`type HttpResponse` |
| [Instant 与 Duration：时间点和时间长度](instant-and-duration.md) | now 是时间戳还是时长？last_refill 为什么减去余数？ | 基础部分可直接阅读；令牌桶应用延至阶段 10。 | [src/token_bucket.rs](../../../../src/token_bucket.rs)：`refill` |
| [表达式、返回值与分号](expressions-and-semicolons.md) | 为什么 match 后加分号可能把结果变成 ()？ | 无；认识整数、布尔值和函数输入输出即可。 | [src/main.rs](../../../../src/main.rs)：策略初始化；[src/gateway.rs](../../../../src/gateway.rs)：selected_idex |
| [String 与 str：拥有文本还是借用文本](string-and-str.md) | &String 和 &str 是同一种类型吗？ | [struct、enum、impl 与 Self](struct-enum-and-impl.md)基础部分；项目应用另按节标题补前置。 | [src/worker.rs](../../../../src/worker.rs)：`Worker::id` |
| [数组、切片、Vec 与 usize](arrays-slices-and-vec.md) | 为何借用切片而不接管 Vec？ | [共享借用与可变借用](../ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：`healthy_worker_indices` |
| [迭代时借用还是移动](iteration-and-collection.md) | 迭代时分别取得借用、可变借用还是拥有值？ | [数组、切片、Vec 与 usize](arrays-slices-and-vec.md) · [共享借用与可变借用](../ownership/borrowing.md) | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`WorkerRegistry::snapshot` |
