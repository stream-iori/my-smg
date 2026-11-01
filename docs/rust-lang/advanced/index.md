# 进阶与扩展

[总入口](../index.md) · [问题索引](../questions.md)

这些主题不作为基础阅读的前置。thiserror/clap 当前未引入；上游材料仅在独立实现和验收之后对照。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [流式响应的内存控制与背压](streaming-memory.md) | 怎样限制单条、队列字节和活跃流，防止慢客户端导致积累？ | [SSE 调度](../walkthroughs/sse-scheduling.md)、[资源生命周期](../concepts/ownership/drop-and-raii.md) | [src/bin/fake-worker.rs](../../../src/bin/fake-worker.rs)：`chat_stream`；阶段 11 的设计与后续验收 |
| [Java 泛型与 PECS：变型的入门对照](java-variance.md) | 为什么 List<Cat> 不能作为 List<Animal>？ | Java 泛型基础；不要求先学习 Rust trait 或生命周期。 | [src/policy.rs](../../../src/policy.rs)：`泛型策略与 Java 对照` |
| [协变与不变：从 Java 泛型到 Rust 引用](variance.md) | 外层借用可以缩短，为何不能随意替换内层引用类型？ | [Java 泛型与 PECS：变型的入门对照](java-variance.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | [src/worker.rs](../../../src/worker.rs)：`InFlightGuard<'a>` |
| [两层引用：外层生命周期与内层类型](reference-variance.md) | 为什么缩短借用时间不等于允许写入任意短引用？ | [协变与不变：从 Java 泛型到 Rust 引用](variance.md) | [src/worker.rs](../../../src/worker.rs)：`InFlightGuard<'a>` |
| [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) | fn(&str) 与某个固定 fn(&'a str) 能直接画等号吗？ | [协变与不变：从 Java 泛型到 Rust 引用](variance.md) | [src/gateway/retry.rs](../../../src/gateway/retry.rs)：`Future 约束的阅读前置` |
| [历史案例：生命周期约束检查遗漏](compiler-case-study.md) | 变型规则与编译器检查遗漏为什么是两回事？ | [协变与不变：从 Java 泛型到 Rust 引用](variance.md) | [docs/rust-lang/advanced/variance.md](variance.md)：`历史实验的上下文` |
| [设计对照：责任、机制与策略](design-decisions.md) | 什么情况下底层保留具体错误、高层再决定响应？ | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | [src/config.rs](../../../src/config.rs)：`加载和校验的责任边界` |
| [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md) | 当前 guard 的借用设计与上游共享拥有设计有什么不同？ | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md)基础部分；项目应用另按节标题补前置。 | [learning/04-source-map.md](../../../learning/04-source-map.md)：`上游对照入口` |
| [扩展：thiserror 的错误派生](thiserror.md) | thiserror 自动生成能力，是否等于自动处理错误？ | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) · [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) · [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../src/config.rs)：`ConfigLoadError` |
| [扩展：clap 的参数派生](clap.md) | 派生参数解析和业务校验有什么不同？ | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md)基础部分；项目应用另按节标题补前置。 | [src/bin/fake-worker.rs](../../../src/bin/fake-worker.rs)：`env::args` |
| [扩展：支付对象的所有权选择](payment-ownership.md) | 只读保存能否使用 &Payment，为什么？ | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | [src/worker.rs](../../../src/worker.rs)：`对照拥有值与借用的差异` |
| [扩展：支付值对象与 Copy](payment-values.md) | 小值对象与可变业务实体应如何区分？ | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | [src/circuit_breaker.rs](../../../src/circuit_breaker.rs)：`CircuitState 的 Copy 对照` |
| [扩展：Clone 与持久化一致性](payment-consistency.md) | 禁止 Clone 是否能替代乐观锁？ | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | [src/circuit_breaker/gate.rs](../../../src/circuit_breaker/gate.rs)：`对照 generation 的版本校验` |
