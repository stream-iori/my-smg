# 错误、转换与传播

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Option 与 Result：有无与成败](option-and-result.md) | Option 与 Result 分别表达什么？ | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md)基础部分；项目应用另按节标题补前置。 | [src/policy.rs](../../../../src/policy.rs)：Option<usize>；[src/config.rs](../../../../src/config.rs)：配置加载的 Result |
| [From 与 Into：为什么有两个转换入口](from-and-into.md) | into 是强制转换、clone，还是提前返回？ | [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) | [src/config.rs](../../../../src/config.rs)：`impl From<std::io::Error> for ConfigLoadError` |
| [? 与 ??：成功继续，失败提前返回](question-mark.md) | Result 与 Option 为什么不能直接混用问号？ | [Option 与 Result：有无与成败](option-and-result.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`operation().await?` |
| [unwrap、expect 与可恢复失败](unwrap-and-expect.md) | 正式配置加载为什么用 Result 而不是 expect？ | [Option 与 Result：有无与成败](option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`expect("managed task should own its handle")` |
| [TryFrom、TryInto 与可能失败的转换](fallible-conversions.md) | parse 和 try_into 分别转换什么来源？ | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Option 与 Result：有无与成败](option-and-result.md) · [类型推断、parse、collect 与空 Vec](../types/type-inference.md)基础部分；项目应用另按节标题补前置。 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`parse::<u64>` |
| [Debug、Display 与 Formatter](debug-display-and-formatter.md) | 为什么 fmt 接收 &mut Formatter<'_>？ | [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../../src/config.rs)：`impl Display for ConfigLoadError` |
| [错误枚举、Error 与 source](error-and-source.md) | From 会拿走错误，source 会拿走吗？ | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Debug、Display 与 Formatter](debug-display-and-formatter.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../../src/config.rs)：`ConfigLoadError / Error::source` |
| [转换与所有权：into 到底消费什么](conversion-and-ownership.md) | 为什么 String.into 和 str 引用.into 对原值的影响不同？ | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Move 与部分移动](../ownership/move-and-partial-move.md)基础部分；项目应用另按节标题补前置。 | [src/main.rs](../../../../src/main.rs)：`error.into()` |
| [??：逐层拆开 Result 与 Option](nested-results.md) | Ok(Err(error)) 与 Err(error) 分别在哪一步返回？ | [? 与 ??：成功继续，失败提前返回](question-mark.md) | [src/gateway.rs](../../../../src/gateway.rs)：`timeout 的两层结果` |
| [Option 的借用、取值与转换：as_ref、take、map](option-access-and-take.md) | 借用、取走与 map 转换分别怎样影响原值？ | [Option 与 Result](option-and-result.md) · [借用](../ownership/borrowing.md) · [Box](../sharing/box.md)基础部分 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：handle.take()；阶段 12 请求 ID 的 map 转换 |
