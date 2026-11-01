# 泛型与类型能力

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [类型推断、parse、collect 与空 Vec](type-inference.md) | 为什么空 Vec 单独写出来时编译器可能无法知道元素类型？ | [数组、切片、Vec 与 usize](../values/arrays-slices-and-vec.md)基础部分；项目应用另按节标题补前置。 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`parse::<u64>` |
| [泛型、类型推断与 turbofish](generics-and-inference.md) | 类型参数 T 与运行时参数 value 有什么区别？ | [数组、切片、Vec 与 usize](../values/arrays-slices-and-vec.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`ManagedTask<T>` |
| [Trait：类型需要提供什么能力](traits-and-bounds.md) | 为什么实现了 Policy 之后还可能需要把 trait 引入作用域？ | [泛型、类型推断与 turbofish](generics-and-inference.md) · [共享借用与可变借用](../ownership/borrowing.md) · [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：`Policy / impl Policy for RoundRobin` |
| [derive 与 Default：编译器替你生成什么](derive-and-default.md) | derive(Default) 与手写 new 是否必然一致？ | [Trait：类型需要提供什么能力](traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`#[derive(Default)]` |
| [PartialEq 与 Eq：怎样定义相等](equality.md) | 两个 Arc 指向的值相等是否表示同一个对象？ | [Trait：类型需要提供什么能力](traits-and-bounds.md) · [derive 与 Default：编译器替你生成什么](derive-and-default.md)基础部分；项目应用另按节标题补前置。 | [src/worker.rs](../../../../src/worker.rs)：`HealthStatus` |
| [相等、排序与哈希](comparison-and-hash.md) | 为什么浮点数会阻止某些 Eq 和 Ord 派生？ | [PartialEq 与 Eq：怎样定义相等](equality.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../../src/config.rs)：`#[derive(Debug, Deserialize, PartialEq, Eq)]` |
| [类型别名与新类型](aliases-and-newtypes.md) | 为什么类型别名不需要 into 就能赋给原类型？ | [元组、元组结构体与 unit](../values/tuples-and-unit.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`HttpResponse` |
| [静态与动态分发](static-and-dynamic-dispatch.md) | Box<dyn Policy> 中 Box 与 dyn 各负责什么？ | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [Box：独占拥有堆上对象](../sharing/box.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：`select_with_policy / select_with_dynamic_policy` |
| [TypeId 与 Any：按类型查找和恢复值](type-id-and-any.md) | 类型如何作为 Map 的 key，类型擦除后怎样检查具体类型？ | [静态与动态分发](static-and-dynamic-dispatch.md) · [相等、排序与哈希](comparison-and-hash.md) | [src/main.rs](../../../../src/main.rs)：`RequestId` 的请求扩展 |
| [哪些 trait 能作为 dyn Trait](dyn-compatibility.md) | 为什么不是任意 trait 都能写成 dyn Trait？ | [静态与动态分发](static-and-dynamic-dispatch.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：`trait Policy` |
| [关联类型：实现确定自己的 Output](associated-types.md) | Output = u32 是转换结果还是检查已有实现？ | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`Fut: Future<Output = Result<HttpResponse, HttpResponse>>` |
| [闭包与 Fn、FnMut、FnOnce](closures-and-fn.md) | FnOnce 是否代表运行时最多只能调用一次？ | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [共享借用与可变借用](../ownership/borrowing.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`wait_until` |
