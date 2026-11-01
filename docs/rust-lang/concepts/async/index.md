# Rust 异步机制

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Future 与 await：创建、暂停和继续](future-and-await.md) | 创建 Future 与执行它是同一时间吗？ | [泛型、类型推断与 turbofish](../types/generics-and-inference.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) · [元组、元组结构体与 unit](../values/tuples-and-unit.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`operation().await` |
| [async move：把哪些值交给 Future](async-capture.md) | move 进去的是对象还是引用，要看什么？ | [Future 与 await：创建、暂停和继续](future-and-await.md) · [闭包与 Fn、FnMut、FnOnce](../types/closures-and-fn.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`operation 闭包` |
