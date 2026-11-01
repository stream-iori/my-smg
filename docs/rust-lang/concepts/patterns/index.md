# 模式与控制流

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [match 分支与守卫的执行顺序](match-and-guards.md) | 两个 Err(error) 为什么不会都执行？ | [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`TestServer::stop` |
| [if let 与 let else：成功值在哪里可用](if-let-and-let-else.md) | else 中为什么不能使用这次绑定的 index？ | [Option 与 Result：有无与成败](../errors/option-and-result.md) · [match 分支与守卫的执行顺序](match-and-guards.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`let Some(index)` |
| [解构：从 State、结构体和元组中取值](destructuring.md) | State 是结构体、表达式还是模式？ | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md) · [元组、元组结构体与 unit](../values/tuples-and-unit.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：State(state) |
| [ref、引用模式与解引用](reference-patterns.md) | 为什么 Drop 里只借用 handle 而不移走？ | [共享借用与可变借用](../ownership/borrowing.md) · [match 分支与守卫的执行顺序](match-and-guards.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`if let Some(handle) = &self.handle` |
