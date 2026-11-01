# 文档迁移与维护记录

[返回总入口](../index.md)

## 本次整理约定

- 以实施前工作区的 21 篇正文为基线，包括未提交的新解释。
- 每个原章节在下表登记归属；归并表示同一知识只保留代表解释，不保留旧导航和重复总结。
- 旧长文直接替换，不维护历史外部链接。新链接使用文件路径，避免依赖 HTML 锚点。
- 不修改项目源码、测试、依赖和课程阶段；不自动提交。

## 写作模板

结论 → 前置/源码 → 真实摘录或明确的独立例子 → 类型/状态/时序 → 对比边界 → 自查与关联。
一篇解决一个问题，通常 80～200 行；短小不靠删除必要解释，超过 250 行检查是否要进一步拆分。

## 迁移清单

| 原文件 / 原行 | 原章节 | 新归属 | 处理 |
|---|---|---|---|
| `borrowed-slices-and-vec-type-annotation.md:1` | 标题与导读 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 重写为结论、前置与导航 |
| `borrowed-slices-and-vec-type-annotation.md:3` | 一句话 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 归并到新结论/导航 |
| `borrowed-slices-and-vec-type-annotation.md:19` | `&[Worker]` 拆开看 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:55` | 为什么参数不写成 `Vec<Worker>` | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:79` | `[Worker]` 与 `[Worker; N]` 不同 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:90` | turbofish：给泛型指定类型 | [类型推断、parse、collect 与空 Vec](../concepts/types/type-inference.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:118` | `Vec::<usize>::new()` 拆开看 | [类型推断、parse、collect 与空 Vec](../concepts/types/type-inference.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:141` | 为什么空 `Vec` 经常需要类型提示 | [类型推断、parse、collect 与空 Vec](../concepts/types/type-inference.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:171` | `parse::<u64>()`：给泛型方法指定目标类型 | [类型推断、parse、collect 与空 Vec](../concepts/types/type-inference.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:226` | pending：泛型类型与函数调用 | [Future 与 await：创建、暂停和继续](../concepts/async/future-and-await.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:284` | 对照：输入和输出都使用泛型参数 | [泛型、类型推断与 turbofish](../concepts/types/generics-and-inference.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:339` | 为什么索引使用 `usize` | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:351` | 放回当前场景 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 迁移并校订 |
| `borrowed-slices-and-vec-type-annotation.md:384` | 一句话记忆 | [数组、切片、Vec 与 usize](../concepts/values/arrays-slices-and-vec.md) | 归并到新结论/导航 |
| `ownership-move-and-borrowing.md:1` | 标题与导读 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 重写为结论、前置与导航 |
| `ownership-move-and-borrowing.md:3` | 一句话 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 归并到新结论/导航 |
| `ownership-move-and-borrowing.md:23` | 所有权解决什么问题 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:50` | move：转移所有权，不复制数据 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:72` | move 不只发生在赋值时 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:117` | 函数返回值也能转移所有权 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:154` | 借用 `&T`：暂时读取，不取得所有权 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:192` | String as str vs borrow | [String 与 str：拥有文本还是借用文本](../concepts/values/string-and-str.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:219` | 为什么 `client.post(...)` 两种写法都可以？ | [Client 与请求构建](../libraries/reqwest/client-and-request.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:243` | 可变借用 `&mut T`：暂时独占并修改 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:259` | 多个共享借用，或者一个可变借用 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:305` | 借用会在最后一次使用后结束 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:334` | guard 借用 Worker：生命周期如何防止先销毁被借用者 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:382` | 方法中的 `self`、`&self`、`&mut self` | [self、mut self、&self 与 &mut self](../concepts/ownership/method-receivers.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:489` | 消费自身与 Option::take：ManagedTask 的例子 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:493` | 1. `mut self`：对象交给方法，方法内部可以修改它 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:511` | 2. 为什么调用方不需要 `let mut task` | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:527` | 3. 异步方法：交出对象和执行方法体是两个时间点 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:558` | 4. `Option::take`：取出句柄，留下合法状态 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:574` | 5. 用 Box 统一理解 `as_ref()`、`as_deref()` 与 `take()` | [Option、Result 与容器中的借用和取值](../concepts/errors/option-and-result.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:647` | 6. 回到 ManagedTask：句柄能借用、能取走，但不能解引用 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:673` | `Copy` 与 `Clone` 是 move 的两个特殊分支 | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 同主题归并，保留代表例子与关键差异 |
| `ownership-move-and-borrowing.md:706` | 结构体字段也可以发生部分 move | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:753` | 容器迭代时的所有权 | [迭代时借用还是移动](../concepts/values/iteration-and-collection.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:787` | `Arc` 是共享所有权，不是绕过所有权 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:818` | 无状态支付请求中的完整例子 | [扩展：支付对象的所有权选择](../advanced/payment-ownership.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:861` | 常见编译错误 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 归并到新结论/导航 |
| `ownership-move-and-borrowing.md:863` | `use of moved value` | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:879` | `cannot borrow ... as mutable` | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:893` | `cannot borrow ... as mutable because it is also borrowed as immutable` | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:898` | 决策表 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `ownership-move-and-borrowing.md:910` | 一句话记忆 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 归并到新结论/导航 |
| `traits-dispatch-and-smart-pointers.md:1` | 标题与导读 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) | 重写为结论、前置与导航 |
| `traits-dispatch-and-smart-pointers.md:3` | 一句话 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) | 归并到新结论/导航 |
| `traits-dispatch-and-smart-pointers.md:16` | 从两个策略开始 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:50` | Trait 与 Java interface 的类比 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:86` | 为什么共同签名使用 `&mut self` | [self、mut self、&self 与 &mut self](../concepts/ownership/method-receivers.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:112` | 泛型结构体与泛型 impl | [泛型、类型推断与 turbofish](../concepts/types/generics-and-inference.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:149` | 泛型：静态分发 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:177` | 编译器如何处理泛型调用 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:195` | 静态分发的特点 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:206` | `impl Trait` 与静态分发 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:232` | `dyn Trait`：动态分发 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:260` | 为什么通常不能单独写 `dyn Policy` | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:282` | `&dyn Policy`：只借用，不拥有 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:310` | `Box<dyn Policy>`：独占所有权与运行时切换 | [Box：独占拥有堆上对象](../concepts/sharing/box.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:346` | 何时适合 `Box<dyn Trait>` | [Box：独占拥有堆上对象](../concepts/sharing/box.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:353` | `Arc<dyn Policy>`：共享所有权 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:387` | `Box`、`Rc`、`Arc`、`Weak` 如何选择 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 同主题归并，保留代表例子与关键差异 |
| `traits-dispatch-and-smart-pointers.md:411` | 动态分发与工厂方法 | [代码导读：轮询选择与策略状态](../walkthroughs/policy-selection.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:437` | Trait object 必须满足 dyn compatibility | [哪些 trait 能作为 dyn Trait](../concepts/types/dyn-compatibility.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:468` | `Send` 和 `Sync` 是另一组约束 | [Send 与 Sync：哪些值能跨线程](../concepts/sharing/send-and-sync.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:485` | `my-smg`：配置驱动的共享策略 | [代码导读：轮询选择与策略状态](../walkthroughs/policy-selection.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:528` | 静态分发与动态分发对照 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:544` | 常见编译错误 | [静态与动态分发](../concepts/types/static-and-dynamic-dispatch.md) | 归并到新结论/导航 |
| `traits-dispatch-and-smart-pointers.md:546` | `the size for values of type dyn Trait cannot be known` | [哪些 trait 能作为 dyn Trait](../concepts/types/dyn-compatibility.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:557` | 找不到 trait 提供的方法 | [Trait：类型需要提供什么能力](../concepts/types/traits-and-bounds.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:565` | 不能把 `&` 传给要求 `&mut` 的参数 | [共享借用与可变借用](../concepts/ownership/borrowing.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:574` | `Arc` 中的值不能可变借用 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 迁移并校订 |
| `traits-dispatch-and-smart-pointers.md:579` | 当前学习顺序 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 归并到新结论/导航 |
| `traits-dispatch-and-smart-pointers.md:593` | 一句话记忆 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 归并到新结论/导航 |
| `common-derive.md:1` | 标题与导读 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 重写为结论、前置与导航 |
| `common-derive.md:3` | 一句话 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 归并到新结论/导航 |
| `common-derive.md:23` | 两类 `derive` | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 归并到新结论/导航 |
| `common-derive.md:25` | Rust 内置 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:34` | 第三方过程宏 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:62` | 最常见组合 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:77` | `Debug`：开发时查看值 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `common-derive.md:100` | 理解 `Clone` / `Copy` 前，先理解 move | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 同主题归并，保留代表例子与关键差异 |
| `common-derive.md:128` | `Clone`：明确说“请再复制一份” | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:158` | `#[derive(Clone)]` 做了什么 | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:184` | `clone()` 不一定是深拷贝 | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:209` | `Copy`：使用旧值时自动复制 | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:232` | 给自定义类型派生 `Copy` | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:263` | 传参时也会触发 move 或 Copy | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:288` | `Clone` 与 `Copy` 对照 | [Copy 与 Clone：复制值还是共享句柄](../concepts/ownership/copy-and-clone.md) | 迁移并校订 |
| `common-derive.md:310` | 无状态支付微服务中的 `Clone` / `Copy` | [扩展：支付值对象与 Copy](../advanced/payment-values.md) | 迁移并校订 |
| `common-derive.md:325` | “无状态”不等于没有业务状态 | [扩展：支付值对象与 Copy](../advanced/payment-values.md) | 迁移并校订 |
| `common-derive.md:360` | `Copy`：适合小型、固定大小的值对象 | [扩展：支付值对象与 Copy](../advanced/payment-values.md) | 迁移并校订 |
| `common-derive.md:427` | Request、Command 和 DTO：一个消费者优先 move | [扩展：支付对象的所有权选择](../advanced/payment-ownership.md) | 迁移并校订 |
| `common-derive.md:476` | Entity / Aggregate 可以 `Clone`，但它只产生内存快照 | [扩展：Clone 与持久化一致性](../advanced/payment-consistency.md) | 迁移并校订 |
| `common-derive.md:541` | 支付一致性依靠版本条件，而不是禁止 `Clone` | [扩展：Clone 与持久化一致性](../advanced/payment-consistency.md) | 迁移并校订 |
| `common-derive.md:579` | Domain Event：按分发方式选择 `Clone` 或 `Arc` | [扩展：Clone 与持久化一致性](../advanced/payment-consistency.md) | 迁移并校订 |
| `common-derive.md:613` | Repository client 与数据库 transaction 不一样 | [扩展：Clone 与持久化一致性](../advanced/payment-consistency.md) | 迁移并校订 |
| `common-derive.md:636` | 无状态支付服务中的推荐选择 | [扩展：支付对象的所有权选择](../advanced/payment-ownership.md) | 迁移并校订 |
| `common-derive.md:672` | `Default`：提供默认值 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:712` | `PartialEq`：让类型可以使用 `==` 和 `!=` | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:786` | enum 如何比较 | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:809` | `Eq`：保证“每个值都等于自己” | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:834` | 为什么浮点数只有 `PartialEq`，没有 `Eq` | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:865` | 为什么 `HashMap` 的 key 常要求 `Eq + Hash` | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:882` | 派生比较是否应该包含所有字段 | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:915` | `PartialEq` 与 `Eq` 对照 | [PartialEq 与 Eq：怎样定义相等](../concepts/types/equality.md) | 迁移并校订 |
| `common-derive.md:924` | `PartialOrd` 与 `Ord`：排序 | [相等、排序与哈希](../concepts/types/comparison-and-hash.md) | 迁移并校订 |
| `common-derive.md:943` | `Hash`：作为哈希集合的 key | [相等、排序与哈希](../concepts/types/comparison-and-hash.md) | 迁移并校订 |
| `common-derive.md:970` | Serde：`Serialize` 与 `Deserialize` | [Serialize 与 Deserialize](../libraries/serde/serialize-and-deserialize.md) | 迁移并校订 |
| `common-derive.md:1010` | thiserror：`Error` | [扩展：thiserror 的错误派生](../advanced/thiserror.md) | 迁移并校订 |
| `common-derive.md:1018` | 只使用 Rust 标准库 | [扩展：thiserror 的错误派生](../advanced/thiserror.md) | 迁移并校订 |
| `common-derive.md:1064` | 使用 `thiserror` 的等价写法 | [扩展：thiserror 的错误派生](../advanced/thiserror.md) | 迁移并校订 |
| `common-derive.md:1115` | clap：`Parser`、`Args`、`Subcommand`、`ValueEnum` | [扩展：clap 的参数派生](../advanced/clap.md) | 迁移并校订 |
| `common-derive.md:1146` | 泛型类型的 trait bound | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:1167` | 常见编译错误怎么看 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 归并到新结论/导航 |
| `common-derive.md:1169` | 某个字段没有实现目标 trait | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:1182` | 找不到第三方 derive 宏 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:1194` | `Eq`、`Ord` 被浮点字段阻止 | [相等、排序与哈希](../concepts/types/comparison-and-hash.md) | 迁移并校订 |
| `common-derive.md:1199` | 派生前的检查清单 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 迁移并校订 |
| `common-derive.md:1212` | 一句话记忆 | [derive 与 Default：编译器替你生成什么](../concepts/types/derive-and-default.md) | 归并到新结论/导航 |
| `mutex-guard-deref-and-poisoning.md:1` | 标题与导读 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 重写为结论、前置与导航 |
| `mutex-guard-deref-and-poisoning.md:3` | 结论 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 归并到新结论/导航 |
| `mutex-guard-deref-and-poisoning.md:32` | 为什么需要 `Arc<Mutex<T>>` | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:64` | `lock()` 为什么返回 `Result` | [毒锁：知道状态可能不完整](../concepts/sharing/poisoning.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:112` | 什么是毒锁 | [毒锁：知道状态可能不完整](../concepts/sharing/poisoning.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:157` | `MutexGuard<'_, usize>` 的两个参数 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:184` | `MutexGuard` 为什么能用 `*` 解引用 | [Deref：为什么能写 *guard](../concepts/sharing/deref-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:220` | 读取 | [Deref：为什么能写 *guard](../concepts/sharing/deref-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:230` | 修改 | [Deref：为什么能写 *guard](../concepts/sharing/deref-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:258` | 解引用不一定意味着移动数据 | [Deref：为什么能写 *guard](../concepts/sharing/deref-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:276` | 为什么用额外的大括号 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:312` | 为什么不能持有 Guard 跨越 `.await` | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 迁移并校订 |
| `mutex-guard-deref-and-poisoning.md:350` | 与模式解构、智能指针的联系 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 归并到新结论/导航 |
| `mutex-guard-deref-and-poisoning.md:360` | 一句话记忆 | [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) | 归并到新结论/导航 |
| `permit-state-version-and-lifecycle.md:1` | 标题与导读 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 重写为结论、前置与导航 |
| `permit-state-version-and-lifecycle.md:5` | 核心感悟 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 归并到新结论/导航 |
| `permit-state-version-and-lifecycle.md:21` | 1. Permit 显式保存请求与熔断器的关系 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:53` | 2. 为什么需要状态版本：A、B、C 的例子 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:76` | 3. Arc<()> 保存什么 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:103` | 4. 与 SOLID 和系统设计原则的关系 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:117` | 编译器能帮什么，不能帮什么 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:136` | 5. 可以迁移到哪些场景 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:153` | 6. 当前练习还缺什么 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 迁移并校订 |
| `permit-state-version-and-lifecycle.md:163` | 一句话总结 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 归并到新结论/导航 |
| `axum-routing-handler-and-future.md:1` | 标题与导读 | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 重写为结论、前置与导航 |
| `axum-routing-handler-and-future.md:16` | 1. 按括号层次读代码 | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:50` | 2. 看 post 签名：输入不是返回值 | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:75` | 3. 看 Handler 约束：为什么调用后返回 Future？ | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:124` | 4. 注册路由与处理请求的时间线 | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:152` | 5. MethodRouter 是什么类型？ | [Router 与 Handler：注册和执行是两件事](../libraries/axum/router-and-handler.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:167` | 6. 两个 post 不要混淆 | [Client 与请求构建](../libraries/reqwest/client-and-request.md) | 迁移并校订 |
| `axum-routing-handler-and-future.md:181` | 关联与源码依据 | [Client 与请求构建](../libraries/reqwest/client-and-request.md) | 归并到新结论/导航 |
| `if-let-ref-and-deref.md:1` | 标题与导读 | [if let 与 let else：成功值在哪里可用](../concepts/patterns/if-let-and-let-else.md) | 重写为结论、前置与导航 |
| `if-let-ref-and-deref.md:3` | 结论 | [if let 与 let else：成功值在哪里可用](../concepts/patterns/if-let-and-let-else.md) | 归并到新结论/导航 |
| `if-let-ref-and-deref.md:36` | `if let` 是什么 | [if let 与 let else：成功值在哪里可用](../concepts/patterns/if-let-and-let-else.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:73` | `let ... else`：取出值，失败就提前离开 | [if let 与 let else：成功值在哪里可用](../concepts/patterns/if-let-and-let-else.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:123` | `State(state)`：解构元组结构体 | [解构：从 State、结构体和元组中取值](../concepts/patterns/destructuring.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:187` | 为什么 `AppState` 需要 `Clone` | [State：框架怎样准备共享状态](../libraries/axum/state.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:207` | 字段借用与 `match` 表达式 | [表达式、返回值与分号](../concepts/values/expressions-and-semicolons.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:264` | match 分支守卫：模式后面还可以加条件 | [match 分支与守卫的执行顺序](../concepts/patterns/match-and-guards.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:313` | 实际类型链 | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 上游结构示例隔离到扩展 |
| `if-let-ref-and-deref.md:339` | 为什么使用 `*` | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 上游结构示例隔离到扩展 |
| `if-let-ref-and-deref.md:361` | 为什么使用 `ref` | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 上游结构示例隔离到扩展 |
| `if-let-ref-and-deref.md:396` | 推荐写法 | [ref、引用模式与解引用](../concepts/patterns/reference-patterns.md) | 归并到新结论/导航 |
| `if-let-ref-and-deref.md:398` | 写法一：使用 `as_ref()` | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 上游结构示例隔离到扩展 |
| `if-let-ref-and-deref.md:421` | 写法二：返回值本来就是 `Option` 时使用 `?` | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `if-let-ref-and-deref.md:437` | 三种写法对照 | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 上游结构示例隔离到扩展 |
| `if-let-ref-and-deref.md:445` | 一句话记忆 | [ref、引用模式与解引用](../concepts/patterns/reference-patterns.md) | 归并到新结论/导航 |
| `variance-lifetimes-generics-and-java.md:1` | 标题与导读 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 重写为结论、前置与导航 |
| `variance-lifetimes-generics-and-java.md:5` | 先抓住一个问题 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:28` | Java：先区分猫和猫列表 | [Java 泛型与 PECS：变型的入门对照](../advanced/java-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:46` | 不变：List<Cat> 不是 List<Animal> | [Java 泛型与 PECS：变型的入门对照](../advanced/java-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:64` | 协变视角：extends，把列表作为动物来源 | [Java 泛型与 PECS：变型的入门对照](../advanced/java-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:87` | 逆变视角：super，把列表作为猫的接收处 | [Java 泛型与 PECS：变型的入门对照](../advanced/java-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:112` | PECS：来源 extends，去向 super | [Java 泛型与 PECS：变型的入门对照](../advanced/java-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:133` | Rust：生命周期提供了另一种替换关系 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:159` | &'static T 和 T: 'static 有什么区别 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:169` | Rust 泛型也有变型 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:190` | 常见类型对 T 的关系 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:208` | 为什么 &mut T 对 T 不变，却对外层生命周期协变 | [两层引用：外层生命周期与内层类型](../advanced/reference-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:233` | 先读懂 &&str 和 *slot | [两层引用：外层生命周期与内层类型](../advanced/reference-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:257` | 把两个生命周期分别标出来 | [两层引用：外层生命周期与内层类型](../advanced/reference-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:271` | 共享引用：里面的长引用可以按短引用读取 | [两层引用：外层生命周期与内层类型](../advanced/reference-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:298` | 可变引用：不能把长期引用槽位当成短期引用槽位 | [两层引用：外层生命周期与内层类型](../advanced/reference-variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:340` | Rust 的逆变：看函数参数 | [高阶生命周期与函数输入逆变](../advanced/higher-ranked-lifetimes.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:365` | Rust trait 约束不是 Java 的类继承 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:401` | 回到讨论起点：issue #164250 | [历史案例：生命周期约束检查遗漏](../advanced/compiler-case-study.md) | 迁移并校订 |
| `variance-lifetimes-generics-and-java.md:433` | 常见误解 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 归并到新结论/导航 |
| `variance-lifetimes-generics-and-java.md:444` | 参考与延伸 | [协变与不变：从 Java 泛型到 Rust 引用](../advanced/variance.md) | 归并到新结论/导航 |
| `tokio-basics-and-task-cancellation.md:1` | 标题与导读 | [Runtime、任务、线程与 Mio](../libraries/tokio/runtime-task-thread.md) | 重写为结论、前置与导航 |
| `tokio-basics-and-task-cancellation.md:5` | 复习路线 | [Runtime、任务、线程与 Mio](../libraries/tokio/runtime-task-thread.md) | 归并到新结论/导航 |
| `tokio-basics-and-task-cancellation.md:19` | 1. 哪些属于 Rust，哪些属于 Tokio | [Runtime、任务、线程与 Mio](../libraries/tokio/runtime-task-thread.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:32` | 2. Future、async 与 await | [Future 与 await：创建、暂停和继续](../concepts/async/future-and-await.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:53` | 3. Runtime、任务和线程 | [Runtime、任务、线程与 Mio](../libraries/tokio/runtime-task-thread.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:74` | 4. spawn、async move 与共享 Worker | [spawn：启动可独立调度的任务](../libraries/tokio/spawn.md) | 归并到新结论/导航 |
| `tokio-basics-and-task-cancellation.md:76` | 4.1 六个概念分别负责什么 | [spawn：启动可独立调度的任务](../libraries/tokio/spawn.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:89` | 4.2 不写 move，也可能移动变量 | [async move：把哪些值交给 Future](../concepts/async/async-capture.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:125` | 4.3 什么时候需要 move | [async move：把哪些值交给 Future](../concepts/async/async-capture.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:156` | 4.4 Future 持有变量，为什么暂停后仍存在 | [async move：把哪些值交给 Future](../concepts/async/async-capture.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:180` | 4.5 Arc、Send 与 static 的边界 | [spawn：启动可独立调度的任务](../libraries/tokio/spawn.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:206` | 4.6 结束、取消与释放 | [取消：abort 之后为什么还要 await](../libraries/tokio/cancellation.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:219` | 5. JoinHandle：不是 Runnable | [JoinHandle：等任务完成，不是任务本体](../libraries/tokio/join-handle.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:241` | 6. sleep、timeout 和 pending | [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | 归并到新结论/导航 |
| `tokio-basics-and-task-cancellation.md:243` | sleep：等待时间，不占住线程 | [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:253` | timeout：给一段异步等待设上限 | [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:280` | pending：故意让任务一直等 | [Future 与 await：创建、暂停和继续](../concepts/async/future-and-await.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:287` | 7. abort 与 Drop：取消测试完整过程 | [取消：abort 之后为什么还要 await](../libraries/tokio/cancellation.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:314` | 7.1 测试重构：ManagedTask 与 TestServer | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 同主题归并，保留代表例子与关键差异 |
| `tokio-basics-and-task-cancellation.md:370` | 8. 与已有文档的联系 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `tokio-basics-and-task-cancellation.md:387` | 一句话复习 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 归并到新结论/导航 |
| `formatter-lifetimes-display-and-error.md:1` | 标题与导读 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 重写为结论、前置与导航 |
| `formatter-lifetimes-display-and-error.md:3` | 一句话 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 归并到新结论/导航 |
| `formatter-lifetimes-display-and-error.md:25` | 先看全貌 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 同主题归并，保留代表例子与关键差异 |
| `formatter-lifetimes-display-and-error.md:52` | `fmt::Formatter<'_>` 拆开看 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:80` | 为什么 Formatter 需要生命周期 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:103` | 生命周期 `'a` 是什么 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:125` | `'_`：让编译器推断生命周期 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:160` | 这里其实有两层借用 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:184` | 为什么是 `&mut Formatter` | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:200` | `Display` 的真实职责 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:226` | 哪些操作会使用 Display | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:240` | `Debug` 与 `Display` | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:267` | `write!` 为什么返回 `fmt::Result` | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:285` | 类型推断在这里做了什么 | [泛型、类型推断与 turbofish](../concepts/types/generics-and-inference.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:311` | 生命周期省略与 `'_` 的区别 | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:333` | `'static` 不等于“这个引用永远存在” | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:357` | `std::error::Error` 的核心结构 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:378` | 空的 `impl Error` 为什么有意义 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:401` | 实现 Error 不会自动做什么 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:415` | 为什么需要统一 Error 接口 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 归并到新结论/导航 |
| `formatter-lifetimes-display-and-error.md:417` | 泛型代码 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:427` | 动态分发 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:438` | 多种错误的统一返回 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:450` | `Error`、`From` 和 `?` 的分工 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 同主题归并，保留代表例子与关键差异 |
| `formatter-lifetimes-display-and-error.md:476` | `From` 与 `source()` 不要混淆 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:546` | 当前 ConfigError 的完整角色 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 同主题归并，保留代表例子与关键差异 |
| `formatter-lifetimes-display-and-error.md:570` | 常见编译错误 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 归并到新结论/导航 |
| `formatter-lifetimes-display-and-error.md:572` | `missing lifetime specifier` | [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:577` | `Formatter` 缺少生命周期参数 | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:581` | `ConfigError` 没有实现 Display | [Debug、Display 与 Formatter](../concepts/errors/debug-display-and-formatter.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:585` | `?` 无法转换错误 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:590` | `source()` 返回的引用生命周期不够长 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `formatter-lifetimes-display-and-error.md:595` | 推荐阅读顺序 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 归并到新结论/导航 |
| `formatter-lifetimes-display-and-error.md:611` | 一句话记忆 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 归并到新结论/导航 |
| `workspace.md:1` | 标题与导读 | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 重写为结论、前置与导航 |
| `workspace.md:3` | 一句话 | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 归并到新结论/导航 |
| `workspace.md:15` | 先用 Maven 对照理解 | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 迁移并校订 |
| `workspace.md:45` | Workspace、Package 与 Crate 的层级 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 迁移并校订 |
| `workspace.md:64` | Cargo 如何识别 library 和 binary | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 迁移并校订 |
| `workspace.md:131` | 为什么使用 Workspace | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 迁移并校订 |
| `workspace.md:157` | 根 `Cargo.toml` | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 迁移并校订 |
| `workspace.md:191` | 统一 package 元数据 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:224` | 统一依赖版本 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:262` | member 之间如何依赖 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:295` | `resolver` 是什么 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:311` | profile 只能在根清单生效 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:331` | 常用命令 | [Workspace 命令：选择构建哪些成员](../concepts/project/workspace-commands.md) | 迁移并校订 |
| `workspace.md:365` | 用 Maven 命令帮助记忆 | [Workspace 命令：选择构建哪些成员](../concepts/project/workspace-commands.md) | 迁移并校订 |
| `workspace.md:379` | 默认构建哪些 member | [Workspace 命令：选择构建哪些成员](../concepts/project/workspace-commands.md) | 迁移并校订 |
| `workspace.md:397` | 如何确认某个 package 属于哪个 Workspace | [Workspace 命令：选择构建哪些成员](../concepts/project/workspace-commands.md) | 迁移并校订 |
| `workspace.md:427` | 常见误区 | [Workspace 的继承、依赖与构建选择](../concepts/project/workspace-configuration.md) | 迁移并校订 |
| `workspace.md:441` | 与模块系统的边界 | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 归并到新结论/导航 |
| `workspace.md:451` | 一句话记忆 | [Workspace：统一管理多个包](../concepts/project/workspace.md) | 归并到新结论/导航 |
| `result-question-mark-and-error-propagation.md:1` | 标题与导读 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 重写为结论、前置与导航 |
| `result-question-mark-and-error-propagation.md:3` | 一句话 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 归并到新结论/导航 |
| `result-question-mark-and-error-propagation.md:16` | 从 JSON 配置解析开始 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:44` | `Result<T, E>` 是什么 | [Option、Result 与容器中的借用和取值](../concepts/errors/option-and-result.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:77` | `?` 的核心行为 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 同主题归并，保留代表例子与关键差异 |
| `result-question-mark-and-error-propagation.md:106` | `?` 近似等价于什么 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:131` | “交给调用者”到底是什么意思 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:167` | 为什么当前函数也必须允许返回错误 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:193` | 为什么成功值需要重新包进 `Ok` | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:223` | 直接返回与使用 `?` 的选择 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 归并到新结论/导航 |
| `result-question-mark-and-error-propagation.md:225` | 直接返回 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:235` | 使用 `?` | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:249` | 多个步骤如何连续传播错误 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:273` | 错误类型不同时发生什么 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:302` | 类型转换：Into、From、TryInto 与 TryFrom | [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:307` | 网关启动错误中的 `.into()` | [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:335` | 常见场景与类型推断 | [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:350` | From 与 Into：同方向转换的两个入口 | [From 与 Into：为什么有两个转换入口](../concepts/errors/from-and-into.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:420` | 所有权与可逆性 | [转换与所有权：into 到底消费什么](../concepts/errors/conversion-and-ownership.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:429` | 可能失败的转换 | [TryFrom、TryInto 与可能失败的转换](../concepts/errors/fallible-conversions.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:446` | 与 `?`、Error 和 source 的关联 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 同主题归并，保留代表例子与关键差异 |
| `result-question-mark-and-error-propagation.md:461` | `?` 也能用于 `Option` | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:499` | Option 与 Result 不能直接混用 `?` | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:535` | 连续两个问号：逐层处理嵌套结果 | [??：逐层拆开 Result 与 Option](../concepts/errors/nested-results.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:539` | Result 套 Result：`handle.await??` | [??：逐层拆开 Result 与 Option](../concepts/errors/nested-results.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:572` | Option 套 Option：同样逐层处理 | [??：逐层拆开 Result 与 Option](../concepts/errors/nested-results.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:590` | `?` 与 `unwrap`、`expect` 的区别 | [unwrap、expect 与可恢复失败](../concepts/errors/unwrap-and-expect.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:609` | 解析错误与业务校验错误不同 | [JSON 解析与业务校验](../libraries/serde/json-parsing-and-validation.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:636` | 所有权发生了什么 | [Move 与部分移动](../concepts/ownership/move-and-partial-move.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:649` | 常见误解 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 归并到新结论/导航 |
| `result-question-mark-and-error-propagation.md:651` | `?` 会处理错误 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:656` | `?` 遇到错误会退出整个程序 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:660` | 使用 `?` 后返回类型变成成功类型 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:664` | 所有错误都能自动用 `?` 转换 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:668` | `?` 可以放在任意函数中 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 迁移并校订 |
| `result-question-mark-and-error-propagation.md:672` | 一句话记忆 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 归并到新结论/导航 |
| `modules-visibility-and-crate.md:1` | 标题与导读 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 重写为结论、前置与导航 |
| `modules-visibility-and-crate.md:3` | 先建立对应关系 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 归并到新结论/导航 |
| `modules-visibility-and-crate.md:29` | `crate`：像 JPMS module 一样理解边界 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:56` | `mod`：接近 Java package，但父子关系有实际作用 | [模块声明如何找到文件](../concepts/project/modules-and-files.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:87` | 从 `pub mod gate;` 到 `circuit_breaker/gate.rs`：逐层找文件 | [模块声明如何找到文件](../concepts/project/modules-and-files.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:145` | 把 mod、pub、use 分开看 | [模块声明如何找到文件](../concepts/project/modules-and-files.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:174` | 与 mod.rs 写法的关系 | [模块声明如何找到文件](../concepts/project/modules-and-files.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:201` | 模块路径：从哪里开始找 | [crate、self、super 与 use](../concepts/project/paths-and-imports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:220` | `pub`：对照 Java 的 `public` 与 `exports` 两层检查 | [pub 与受限可见性](../concepts/project/visibility.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:244` | Rust 可见性表 | [pub 与受限可见性](../concepts/project/visibility.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:258` | 多一层模块时：`pub(super)` 与 `pub(in 路径)` | [pub 与受限可见性](../concepts/project/visibility.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:305` | `use`：最接近 Java import | [crate、self、super 与 use](../concepts/project/paths-and-imports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:325` | `pub use`：提供公开入口，Java exports 不会改路径 | [pub use：内部组织与公开入口](../concepts/project/reexports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:347` | 文件布局、模块路径与公开入口可以分别设计 | [pub use：内部组织与公开入口](../concepts/project/reexports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:373` | Java 9 另外三个关键词不要混淆 | [pub use：内部组织与公开入口](../concepts/project/reexports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:383` | 最小记忆图 | [pub use：内部组织与公开入口](../concepts/project/reexports.md) | 归并到新结论/导航 |
| `modules-visibility-and-crate.md:403` | 一个完整例子：把所有概念串起来 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 归并到新结论/导航 |
| `modules-visibility-and-crate.md:407` | 1. 先看文件，再数 package 和 crate | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:431` | 2. 文件如何进入模块树：`mod` 与 `pub mod` | [模块声明如何找到文件](../concepts/project/modules-and-files.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:455` | 3. 用一次访问检查理解全部可见性 | [pub 与受限可见性](../concepts/project/visibility.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:471` | 4. 同一个名字，从不同位置怎么找：路径与 `use` | [crate、self、super 与 use](../concepts/project/paths-and-imports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:487` | 5. 改善对外入口：`pub use` | [pub use：内部组织与公开入口](../concepts/project/reexports.md) | 迁移并校订 |
| `modules-visibility-and-crate.md:508` | 6. 换成 Java 9：同一个设计如何理解 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 同主题归并，保留代表例子与关键差异 |
| `modules-visibility-and-crate.md:544` | 理解检查 | [Package 与 Crate：一个工程里有几棵编译树](../concepts/project/package-and-crate.md) | 归并到新结论/导航 |
| `arc-weak-and-cycle-references.md:1` | 标题与导读 | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 重写为结论、前置与导航 |
| `arc-weak-and-cycle-references.md:3` | 结论 | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 归并到新结论/导航 |
| `arc-weak-and-cycle-references.md:25` | `Arc` 与 `Weak` 的区别 | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:41` | ptr_eq：对象身份与值相等 | [代码导读：许可、状态版本与迟到结果](../walkthroughs/circuit-permit.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:74` | 节点移除与对象销毁是两件事 | [代码导读：Worker 与在途请求的生命周期](../walkthroughs/worker-lifecycle.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:99` | 如果 guard 只保存引用 | [代码导读：Worker 与在途请求的生命周期](../walkthroughs/worker-lifecycle.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:118` | 原项目为什么选择 Arc | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:135` | 为什么两个 `Arc` 会泄漏 | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:161` | 用 `Weak` 断开“反向指针” | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:189` | `upgrade()`：弱引用不能直接使用 | [Weak 与强引用环](../concepts/sharing/weak-and-cycles.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:224` | SGL Model Gateway 中的例子 | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:253` | 最小可运行模型 | [扩展：上游的 Arc 与 Weak 关系](../advanced/upstream-ownership.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:273` | 判断该用哪个 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 迁移并校订 |
| `arc-weak-and-cycle-references.md:286` | 一句话记忆 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) | 归并到新结论/导航 |
| `realization.md:1` | 标题与导读 | [代码导读：Worker 与在途请求的生命周期](../walkthroughs/worker-lifecycle.md) | 重写为结论、前置与导航 |
| `realization.md:5` | 节点删除：分开可选资格与对象存活 | [代码导读：Worker 与在途请求的生命周期](../walkthroughs/worker-lifecycle.md) | 迁移并校订 |
| `realization.md:26` | 引用与 Arc 的选择来自需求，不来自语法喜好 | [代码导读：Worker 与在途请求的生命周期](../walkthroughs/worker-lifecycle.md) | 迁移并校订 |
| `realization.md:34` | 可复用的思考顺序 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `realization.md:43` | 关联阅读 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 归并到新结论/导航 |
| `type-aliases-and-associated-types.md:1` | 标题与导读 | [类型别名与新类型](../concepts/types/aliases-and-newtypes.md) | 重写为结论、前置与导航 |
| `type-aliases-and-associated-types.md:16` | 1. type HttpResponse：别名，不是新类型 | [类型别名与新类型](../concepts/types/aliases-and-newtypes.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:47` | 2. 同一个别名可以出现在 Ok 和 Err 中 | [代码导读：重试与总期限](../walkthroughs/retry-and-timeout.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:75` | 3. Output：Future 的关联类型 | [关联类型：实现确定自己的 Output](../concepts/types/associated-types.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:106` | 怎样显式声明输出类型 | [关联类型：实现确定自己的 Output](../concepts/types/associated-types.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:189` | 4. 为什么写 Output =，而不是 Future<usize>？ | [关联类型：实现确定自己的 Output](../concepts/types/associated-types.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:207` | 5. 拆解当前重试函数的约束 | [代码导读：重试与总期限](../walkthroughs/retry-and-timeout.md) | 迁移并校订 |
| `type-aliases-and-associated-types.md:235` | 6. 放进 timeout 后为什么有两层 Result？ | [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | 同主题归并，保留代表例子与关键差异 |
| `type-aliases-and-associated-types.md:252` | 关联阅读与依据 | [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) | 归并到新结论/导航 |
| `atomic-usize-and-interior-mutability.md:1` | 标题与导读 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 重写为结论、前置与导航 |
| `atomic-usize-and-interior-mutability.md:3` | 结论 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 归并到新结论/导航 |
| `atomic-usize-and-interior-mutability.md:15` | 为什么 `fetch_add` 只需要 `&self` | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 迁移并校订 |
| `atomic-usize-and-interior-mutability.md:31` | 三个操作分别做什么 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 迁移并校订 |
| `atomic-usize-and-interior-mutability.md:51` | `Relaxed` 到底放松了什么 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 迁移并校订 |
| `atomic-usize-and-interior-mutability.md:62` | 与 `Arc`、`Mutex` 的分工 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 迁移并校订 |
| `atomic-usize-and-interior-mutability.md:76` | 继续阅读 | [原子变量与 Relaxed](../concepts/sharing/atomics.md) | 归并到新结论/导航 |
| `desgin-principle.md:1` | 标题与导读 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 重写为结论、前置与导航 |
| `desgin-principle.md:3` | 结论 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 归并到新结论/导航 |
| `desgin-principle.md:19` | 统一例子：配置加载流水线 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 迁移并校订 |
| `desgin-principle.md:44` | 原则总表 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `desgin-principle.md:58` | 1. 失败也是数据 | [Option、Result 与容器中的借用和取值](../concepts/errors/option-and-result.md) | 同主题归并，保留代表例子与关键差异 |
| `desgin-principle.md:83` | 2. 阶段不同，错误语义不同 | [代码导读：配置加载与分层错误](../walkthroughs/configuration-loading.md) | 迁移并校订 |
| `desgin-principle.md:102` | 3. 底层具体，高层统一 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 同主题归并，保留代表例子与关键差异 |
| `desgin-principle.md:130` | 4. 增加语义，但不要丢失证据 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 同主题归并，保留代表例子与关键差异 |
| `desgin-principle.md:172` | 5. 所有权跟随责任 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `desgin-principle.md:196` | 6. 机制与策略分离 | [代码导读：轮询选择与策略状态](../walkthroughs/policy-selection.md) | 迁移并校订 |
| `desgin-principle.md:221` | 7. 机器结构与人类文字分离 | [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) | 同主题归并，保留代表例子与关键差异 |
| `desgin-principle.md:248` | 8. 失败尽早返回 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) | 同主题归并，保留代表例子与关键差异 |
| `desgin-principle.md:278` | 9. 复杂度按真实需求引入 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `desgin-principle.md:301` | 10. 测试重构：抽准备工作，不隐藏行为 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `desgin-principle.md:339` | 设计选择速查 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `desgin-principle.md:353` | 设计检查清单 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 迁移并校订 |
| `desgin-principle.md:367` | 一句话总结 | [设计对照：责任、机制与策略](../advanced/design-decisions.md) | 归并到新结论/导航 |
| `extractor-state-and-trait-dispatch.md:1` | 标题与导读 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 重写为结论、前置与导航 |
| `extractor-state-and-trait-dispatch.md:6` | 先看结论 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 归并到新结论/导航 |
| `extractor-state-and-trait-dispatch.md:35` | 最小例子 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:84` | State 是什么类型 | [解构：从 State、结构体和元组中取值](../concepts/patterns/destructuring.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:110` | trait 怎样描述提取规则 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:152` | dispatch 如何知道该调用哪个实现 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:221` | 提取和解构是两步 | [用最小模型理解 Extractor](../libraries/axum/extractor-mechanics.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:251` | 和真实 Axum 怎么对应 | [State：框架怎样准备共享状态](../libraries/axum/state.md) | 迁移并校订 |
| `extractor-state-and-trait-dispatch.md:274` | 自查理解 | [State：框架怎样准备共享状态](../libraries/axum/state.md) | 归并到新结论/导航 |
| `extractor-state-and-trait-dispatch.md:285` | 关联阅读与源码 | [State：框架怎样准备共享状态](../libraries/axum/state.md) | 归并到新结论/导航 |
| `closures-and-fn-traits.md:1` | 标题与导读 | [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) | 重写为结论、前置与导航 |
| `closures-and-fn-traits.md:5` | 先理解一个调用 | [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) | 迁移并校订 |
| `closures-and-fn-traits.md:32` | Fn、FnMut 与 FnOnce：区别在调用需要什么 | [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) | 迁移并校订 |
| `closures-and-fn-traits.md:79` | 三个小例子对照 | [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) | 迁移并校订 |
| `closures-and-fn-traits.md:120` | 外层 move 闭包与内层 async move | [async move：把哪些值交给 Future](../concepts/async/async-capture.md) | 迁移并校订 |
| `closures-and-fn-traits.md:152` | 闭包与异步块不是同一种值 | [async move：把哪些值交给 Future](../concepts/async/async-capture.md) | 迁移并校订 |
| `closures-and-fn-traits.md:164` | 与条件等待及测试设计的关系 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 迁移并校订 |
| `closures-and-fn-traits.md:178` | 一句话复习 | [代码导读：ManagedTask 的清理责任](../walkthroughs/task-cleanup.md) | 归并到新结论/导航 |
| `ownership-move-and-borrowing.md:427` | into 的 String 与引用对照例子 | [转换与所有权：into 到底消费什么](../concepts/errors/conversion-and-ownership.md) | 从接收者章节抽出，保留完整例子 |

## 验收记录

| 检查 | 结果 |
|---|---|
| 旧内容覆盖 | 21 篇正文，395 条章节/导读/细分例子记录，全部有新归属 |
| 文档结构 | 113 个页面可解析，306 个表格列数一致，首页不可达页面为 0 |
| 普通相对链接与标题锚点 | 静态检查通过；HTML 锚点已全部移除 |
| 前置知识图 | 93 篇正文的前置依赖无环 |
| 正文长度 | 全部概念/库/导读/扩展正文不超过 250 行；导航与维护清单单独计 |
| 独立例子 | 12 组独立片段编译运行通过；另验证 4 个完整 main 示例，其中 3 个运行通过、1 个按预期编译拒绝 |
| 项目源码 | 通过目标目录外文件哈希对照确认未修改源码、配置、测试或课程文档 |
| Zed 预览 | 未启动 Zed 图形界面；通用 Markdown 解析不等同于已验证 Zed UI |

独立例子覆盖 Rc、RefCell、Option<Box<i32>>、into 的拥有值/引用对照、泛型 identity、Option/Result 转换、嵌套 Option、三种 Future 输出声明和 pending 类型。
完整 main 示例包括提取器最小模型、共享引用变型、生命周期示意及错误写入短引用的反例。
Future 声明示例验证编译和类型约束，不尝试执行永不完成的 pending。抽查页面生成了临时 HTML，不将其视为截图或人工视觉验收。

后续维护时先检查所有相对路径和标题锚点，再检查表格列数、代码围栏与前置关系；源码摘录变更时重新核对对应符号。纯文档调整不重跑网络服务或项目业务测试。


## 2026-10-20：面向 Java 学习者的配套改进

本节追加本轮记录，上一轮迁移清单和验收结果保留。教学实施依据仍为 learning，项目实现由学习者编写。

| 改进 | 主要归属 |
|---|---|
| Java 经验迁移与六组核心差异 | [Java → Rust](../java-to-rust.md) |
| T00～T07 必读、回查、验收后阅读；阶段 3～12 范围与缺口 | [阶段阅读清单](../stage-reading.md) |
| 非 Copy 枚举的匹配是否移动、当前 GatewayState 字段与选中索引 | [表达式](../concepts/values/expressions-and-semicolons.md) |
| 当前 Worker::new 双参数接口及借用冲突反例 | [生命周期](../concepts/ownership/lifetimes.md) |
| Option/Result 基础与容器操作分开 | [基础](../concepts/errors/option-and-result.md)、[借用与取值](../concepts/errors/option-access-and-take.md) |
| 最小解释、基础自查、按阶段延后的项目应用 | 工程、值、所有权、模式、泛型基础篇；Runtime 的 Netty 对照改为可选 |
| 前置与目录同步 | 相等先于排序与哈希，类型推断提前，Java PECS 先于依赖它的变型专题；Workspace 深入内容标为可选 |

旧迁移清单中的 Option 操作章节，本轮的主要讲解位置改为“借用与取值”；历史行号和既往验收记录不作追溯改写。
当前项目接口核对覆盖 Worker::new、id/address、begin_request/InFlightGuard、Policy::select、策略选择函数、GatewayState、main 的策略初始化和 ManagedTask 的清理入口。同步策略签名均明确标为历史或简化示意。

| 本轮验证 | 实际结果 |
|---|---|
| Markdown 结构 | 116 个分层页面解析；表格列数、代码围栏、每页单一一级标题检查通过 |
| 导航 | 所有本地链接目标存在，116 页均从首页可达 |
| 前置依赖 | 依赖图无环；前置限定基础部分，后续应用按各节标注补知识 |
| 正文长度 | 概念、库、导读和扩展正文均不超过 250 行；导航与维护清单单独计 |
| 生命周期反例 | rustc 1.96.0，edition 2024；当前双参数构造后先 drop(worker) 再 drop(guard)，仅报目标诊断 E0505 |
| 生命周期正例 | 仅交换上述两次释放顺序，编译通过 |
| 表达式正反例 | 已有 choose 示例编译通过；末尾 match 后添加分号，仅报目标诊断 E0308 |
| 修改范围 | 对照开始前哈希，源码、测试、清单、依赖锁文件和 learning 无变化 |
| 验证边界 | 编译使用仓库外临时目录，引用学习者已有代码；不新增项目测试或检查脚本，不启动网关业务测试，不自动提交 |

工作期间目录曾由分层布局变为 22 篇旧文；经用户明确要求继续原计划后，使用实施前副本恢复分层结构并重做改进，重新验证最终版本。按用户后续指示忽略旧篇目，旧长文不纳入本轮验收。
本轮未进行 Zed 或浏览器视觉验收，也未逐个编译全文中所有省略片段；编译结果仅指上表四个针对性场景。
