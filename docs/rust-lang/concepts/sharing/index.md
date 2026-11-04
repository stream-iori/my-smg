# 共享拥有与修改

[总入口](../../index.md) · [问题索引](../../questions.md)

按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Box：独占拥有堆上对象](box.md) | Box 拥有权与 dyn 的分发方式能否分开理解？ | [共享借用与可变借用](../ownership/borrowing.md) · [Drop 与 RAII：离开作用域如何收尾](../ownership/drop-and-raii.md) | [src/main.rs](../../../../src/main.rs)：`Box<dyn Policy + Send>` |
| [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) | Arc::clone 是否创建新的 Worker？ | [共享借用与可变借用](../ownership/borrowing.md) · [Drop 与 RAII：离开作用域如何收尾](../ownership/drop-and-raii.md) | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`WorkerRegistry::get` |
| [Weak 与强引用环](weak-and-cycles.md) | 弱引用还在时数据一定在吗？ | [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md)基础部分；项目应用另按节标题补前置。 | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`workers: HashMap<String, Arc<Worker>>` |
| [RefCell：单线程里的运行时借用检查](refcell.md) | 只在单线程运行就能随意同时存在多个 &mut 吗？ | [共享借用与可变借用](../ownership/borrowing.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md)基础部分；项目应用另按节标题补前置。 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`failures_remaining` |
| [Mutex 与 Guard：锁何时释放](mutex-and-guards.md) | 额外的大括号如何限制持锁时间？ | [共享借用与可变借用](../ownership/borrowing.md) · [Drop 与 RAII：离开作用域如何收尾](../ownership/drop-and-raii.md) · [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`state.registry.lock()` |
| [毒锁：知道状态可能不完整](poisoning.md) | into_inner 与 clear_poison 是否等同于恢复正确状态？ | [Mutex 与 Guard：锁何时释放](mutex-and-guards.md)基础部分；项目应用另按节标题补前置。 | [src/circuit_gate.rs](../../../../src/circuit_gate.rs)：`CircuitGate::lock_state` |
| [Deref：为什么能写 *guard](deref-and-guards.md) | *guard 是取值、复制，还是写入位置？ | [Mutex 与 Guard：锁何时释放](mutex-and-guards.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`*remaining -= 1` |
| [原子变量与 Ordering](atomics.md) | fetch_add 返回什么？独立计数与发布数据应怎样选择内存顺序？ | [共享借用与可变借用](../ownership/borrowing.md) · [Copy 与 Clone：复制值还是共享句柄](../ownership/copy-and-clone.md)基础部分；项目应用另按节标题补前置。 | [src/worker.rs](../../../../src/worker.rs)：`fetch_add / fetch_sub`；[src/main.rs](../../../../src/main.rs)：请求总数 |
| [Send 与 Sync：哪些值能跨线程](send-and-sync.md) | 为什么 Arc<RefCell<T>> 仍不能直接跨线程共享？ | [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Mutex 与 Guard：锁何时释放](mutex-and-guards.md) · [Trait：类型需要提供什么能力](../types/traits-and-bounds.md)基础部分；项目应用另按节标题补前置。 | [src/gateway.rs](../../../../src/gateway.rs)：`Mutex<Box<dyn Policy + Send>>` |
