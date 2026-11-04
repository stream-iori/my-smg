# 按问题查找

[返回总入口](index.md)

阅读顺序见[阶段清单](stage-reading.md)，Java 对照见[迁移地图](java-to-rust.md)。先按问题进入一篇短文；需要补前置时再沿文章顶部链接阅读。

## 工程与模块

| 问题 | 阅读入口 |
|---|---|
| 为什么 main.rs 不能访问库的 pub(crate) 项？ | [Package 与 Crate：一个工程里有几棵编译树](concepts/project/package-and-crate.md) |
| circuit_breaker.rs 与 circuit_breaker/ 是两个模块吗？ | [模块声明如何找到文件](concepts/project/modules-and-files.md) |
| super 指父类还是父模块？ | [crate、self、super 与 use](concepts/project/paths-and-imports.md) |
| 祖父模块能否访问 pub(super) 的函数？ | [pub 与受限可见性](concepts/project/visibility.md) |
| 公开入口变短是否意味着目录变浅？ | [pub use：内部组织与公开入口](concepts/project/reexports.md) |
| edition、rust-version 与 package version 分别管什么？ | [edition：语言规则与编译器版本](concepts/project/edition.md) |
| 当前 my-smg 是 workspace 成员集合还是一个 package？ | [Workspace：统一管理多个包](concepts/project/workspace.md) |
| workspace.dependencies 是否自动添加依赖？ | [Workspace 的继承、依赖与构建选择](concepts/project/workspace-configuration.md) |
| -p 接受目录名还是 package.name？ | [Workspace 命令：选择构建哪些成员](concepts/project/workspace-commands.md) |

## 类型、值与容器

| 问题 | 阅读入口 |
|---|---|
| Worker::new 与 worker.id 分别如何调用？ | [struct、enum、impl 与 Self](concepts/values/struct-enum-and-impl.md) |
| () 与 (StatusCode, String) 携带多少信息？ | [元组、元组结构体与 unit](concepts/values/tuples-and-unit.md) |
| Instant、Duration 与 Unix 时间戳有什么区别？ | [Instant 与 Duration：时间点和时间长度](concepts/values/instant-and-duration.md) |
| Duration::new 为什么把纳秒拆成秒与余数，而不向上取整？ | [Duration 的单位与构造](concepts/values/instant-and-duration.md#duration-的单位与构造) |
| last_refill 为什么减去 remainder？为什么不定时补令牌？ | [令牌桶时间计算](concepts/values/instant-and-duration.md#项目应用阶段-10-的令牌桶时间计算) |
| 为什么 match 后加分号可能把结果变成 ()？ | [表达式、返回值与分号](concepts/values/expressions-and-semicolons.md) |
| &String 和 &str 是同一种类型吗？ | [String 与 str：拥有文本还是借用文本](concepts/values/string-and-str.md) |
| 为什么同步函数借用切片，而不直接接管 Vec？ | [数组、切片、Vec 与 usize](concepts/values/arrays-slices-and-vec.md) |
| iter、iter_mut 与拥有型 Vec 的 into_iter 分别得到什么？ | [迭代时借用还是移动](concepts/values/iteration-and-collection.md) |

## 所有权、借用与释放

| 问题 | 阅读入口 |
|---|---|
| 为什么未移动的字段有时还能用，而整个对象不能用？ | [Move 与部分移动](concepts/ownership/move-and-partial-move.md) |
| 为什么 policy 是 &mut，而 workers 是共享切片？ | [共享借用与可变借用](concepts/ownership/borrowing.md) |
| 调用方不写 let mut，为什么仍能调用 mut self 方法？ | [self、mut self、&self 与 &mut self](concepts/ownership/method-receivers.md) |
| `&self` 和 `self: &Arc<Self>` 都是借用吗？为什么方法要区分它们？ | [显式接收者与返回值关系](concepts/ownership/method-receivers.md#阶段-11self-与-self-arcself) |
| 变量是 `Arc<Worker>`，为什么可以直接调用接收 `&Arc<Worker>` 的方法？ | [点号调用时的自动借用](concepts/ownership/method-receivers.md#阶段-11self-与-self-arcself) |
| Arc::clone 和 String::clone 的成本与效果相同吗？ | [Copy 与 Clone：复制值还是共享句柄](concepts/ownership/copy-and-clone.md) |
| Option 的 clone 和 cloned 分别复制什么？为什么保存响应头需要 cloned？ | [简单代码与响应头应用](concepts/ownership/copy-and-clone.md#option-的-clone-与-cloned) |
| guard 中的 &'a Worker 限制了什么？ | [生命周期：引用不能比数据活得久](concepts/ownership/lifetimes.md) |
| 为什么借用型 guard 和它的 Drop 要写生命周期，而 Arc 字段的 guard 不用？ | [生命周期参数与共同拥有](concepts/ownership/lifetimes.md#阶段-11借用型-guard-与拥有型-guard) |
| 为什么提前 return 也会让在途计数减一？ | [Drop 与 RAII：离开作用域如何收尾](concepts/ownership/drop-and-raii.md) |
| 为什么并发凭证放在重试闭包外，断路器凭证却放在里面？ | [先确定资源生命周期，再决定作用域](concepts/ownership/drop-and-raii.md#阶段-10先确定资源生命周期再决定作用域) |
| 资源生命周期设计与单一职责、Rust 的 'a 有什么区别？ | [职责、资源区间与引用有效关系](concepts/ownership/drop-and-raii.md#阶段-10先确定资源生命周期再决定作用域) |
| 等待凭证与执行凭证怎样体现关注点分离、单一职责和生命周期设计？ | [排队与执行的职责和作用域](concepts/ownership/drop-and-raii.md#排队与执行同一种机制不同的职责和生命周期) |

## 模式与控制流

| 问题 | 阅读入口 |
|---|---|
| 两个 Err(error) 为什么不会都执行？ | [match 分支与守卫的执行顺序](concepts/patterns/match-and-guards.md) |
| else 中为什么不能使用这次绑定的 index？ | [if let 与 let else：成功值在哪里可用](concepts/patterns/if-let-and-let-else.md) |
| State 是结构体、表达式还是模式？ | [解构：从 State、结构体和元组中取值](concepts/patterns/destructuring.md) |
| 为什么 Drop 里只借用 handle 而不移走？ | [ref、引用模式与解引用](concepts/patterns/reference-patterns.md) |

## 泛型与类型能力

| 问题 | 阅读入口 |
|---|---|
| T 与 value 分别是类型参数还是运行时参数？ | [泛型](concepts/types/generics-and-inference.md) |
| pending::<()>() 中两个 () 各做什么？ | [Future](concepts/async/future-and-await.md) |
| 为什么实现了 Policy 之后还可能需要把 trait 引入作用域？ | [Trait：类型需要提供什么能力](concepts/types/traits-and-bounds.md) |
| Output = u32 是转换结果还是检查已有实现？ | [关联类型：实现确定自己的 Output](concepts/types/associated-types.md) |
| 为什么类型别名不需要 into 就能赋给原类型？ | [类型别名与新类型](concepts/types/aliases-and-newtypes.md) |
| Box<dyn Policy> 中 Box 与 dyn 各负责什么？ | [静态与动态分发](concepts/types/static-and-dynamic-dispatch.md) |
| TypeId 为什么能做 Map 的 key？Any 如何恢复指定类型？ | [TypeId 与 Any](concepts/types/type-id-and-any.md) |
| 为什么不是任意 trait 都能写成 dyn Trait？ | [哪些 trait 能作为 dyn Trait](concepts/types/dyn-compatibility.md) |
| FnOnce 是否代表运行时最多只能调用一次？ | [闭包与 Fn、FnMut、FnOnce](concepts/types/closures-and-fn.md) |
| derive(Default) 与手写 new 是否必然一致？ | [derive 与 Default：编译器替你生成什么](concepts/types/derive-and-default.md) |
| 为什么浮点数会阻止某些 Eq 和 Ord 派生？ | [相等、排序与哈希](concepts/types/comparison-and-hash.md) |
| 两个 Arc 指向的值相等是否表示同一个对象？ | [PartialEq 与 Eq：怎样定义相等](concepts/types/equality.md) |
| 为什么空 Vec 单独写出来时编译器可能无法知道元素类型？ | [类型推断、parse、collect 与空 Vec](concepts/types/type-inference.md) |

## 错误、转换与传播

| 问题 | 阅读入口 |
|---|---|
| Option 与 Result 分别表达什么？ | [Option 与 Result](concepts/errors/option-and-result.md) |
| as_ref、as_deref 和 take 分别留下什么？ | [借用与取值](concepts/errors/option-access-and-take.md) |
| Option 的 map 如何转换 Some？为什么调用后原变量有时还能用？ | [map 与所有权](concepts/errors/option-access-and-take.md#map有值就转换没有就保持-none) |
| RequestId 与 Extension<RequestId> 为什么支持 Copy？ | [Copy 的逐层条件](concepts/ownership/copy-and-clone.md#阶段-12请求-id-的-copy-如何逐层成立) |
| Result 与 Option 为什么不能直接混用问号？ | [? 与 ??：成功继续，失败提前返回](concepts/errors/question-mark.md) |
| into 是强制转换、clone，还是提前返回？ | [From 与 Into：为什么有两个转换入口](concepts/errors/from-and-into.md) |
| parse 和 try_into 分别转换什么来源？ | [TryFrom、TryInto 与可能失败的转换](concepts/errors/fallible-conversions.md) |
| From 会拿走错误，source 会拿走吗？ | [错误枚举、Error 与 source](concepts/errors/error-and-source.md) |
| 为什么 fmt 接收 &mut Formatter<'_>？ | [Debug、Display 与 Formatter](concepts/errors/debug-display-and-formatter.md) |
| 正式配置加载为什么用 Result 而不是 expect？ | [unwrap、expect 与可恢复失败](concepts/errors/unwrap-and-expect.md) |
| Ok(Err(error)) 与 Err(error) 分别在哪一步返回？ | [??：逐层拆开 Result 与 Option](concepts/errors/nested-results.md) |
| 为什么 String.into 和 str 引用.into 对原值的影响不同？ | [转换与所有权：into 到底消费什么](concepts/errors/conversion-and-ownership.md) |

## 共享拥有与修改

| 问题 | 阅读入口 |
|---|---|
| Box 拥有权与 dyn 的分发方式能否分开理解？ | [Box：独占拥有堆上对象](concepts/sharing/box.md) |
| Arc::clone 是否创建新的 Worker？ | [Rc 与 Arc：多个句柄共同拥有数据](concepts/sharing/rc-and-arc.md) |
| `Arc::new` 返回 `Arc<T>` 还是 `&Arc<T>`？与 `&T` 又有什么区别？ | [三种类型与句柄关系](concepts/sharing/rc-and-arc.md#arctarct-与-t-要分开) |
| Arc 的引用计数和 Worker 的在途计数为什么不同？ | [Worker 的两套计数](walkthroughs/worker-lifecycle.md#两套计数分别回答两个问题) |
| 弱引用还在时数据一定在吗？ | [Weak 与强引用环](concepts/sharing/weak-and-cycles.md) |
| 只在单线程运行就能随意同时存在多个 &mut 吗？ | [RefCell：单线程里的运行时借用检查](concepts/sharing/refcell.md) |
| 额外的大括号如何限制持锁时间？ | [Mutex 与 Guard：锁何时释放](concepts/sharing/mutex-and-guards.md) |
| into_inner 与 clear_poison 是否等同于恢复正确状态？ | [毒锁：知道状态可能不完整](concepts/sharing/poisoning.md) |
| *guard 是取值、复制，还是写入位置？ | [Deref：为什么能写 *guard](concepts/sharing/deref-and-guards.md) |
| fetch_add 返回增加前还是增加后的值？ | [原子变量与 Relaxed](concepts/sharing/atomics.md) |
| Ordering 的 Relaxed、Acquire、Release、AcqRel、SeqCst 怎样选择？ | [五种顺序与使用场景](concepts/sharing/atomics.md#ordering-的五种选择) |
| 为什么 Arc<RefCell<T>> 仍不能直接跨线程共享？ | [Send 与 Sync：哪些值能跨线程](concepts/sharing/send-and-sync.md) |

## Rust 异步机制

| 问题 | 阅读入口 |
|---|---|
| 创建 Future 与执行它是同一时间吗？ | [Future 与 await：创建、暂停和继续](concepts/async/future-and-await.md) |
| move 进去的是对象还是引用，要看什么？ | [async move：把哪些值交给 Future](concepts/async/async-capture.md) |
| handler 返回后，SSE 的流由谁持有、谁持续推进？ | [SSE 对象、所有权与唤醒流程](walkthroughs/sse-scheduling.md) |
| SSE 会把三条事件收集齐再发送吗？ | [逐段拉取与 Pending](walkthroughs/sse-scheduling.md#图三三条事件怎样逐段交出) |
| bytes_stream 加 Body::from_stream 为什么不需要收集完整响应？ | [逐块读取与透明转发](walkthroughs/sse-scheduling.md#阶段-11-02不收集完整响应是什么意思) |
| 流式响应为什么仍可能耗尽内存，有界队列是否足够？ | [内存控制、字节预算与背压](advanced/streaming-memory.md) |

## Tokio

| 问题 | 阅读入口 |
|---|---|
| Tokio、Mio 与 Netty EventLoop 分别在哪一层？ | [Runtime、任务、线程与 Mio](libraries/tokio/runtime-task-thread.md) |
| 调用 spawn 后是否需要先 await JoinHandle 才开始运行？ | [spawn：启动可独立调度的任务](libraries/tokio/spawn.md) |
| 丢弃 JoinHandle 是否等于取消任务？ | [JoinHandle：等任务完成，不是任务本体](libraries/tokio/join-handle.md) |
| select! 如何同时等待多个事件？落选会自动 abort 已启动的任务吗？ | [select! 与取消安全](libraries/tokio/select.md) |
| oneshot 如何传一次通知？send 成功表示对方处理完成了吗？ | [oneshot 的发送、接收与关闭](libraries/tokio/oneshot.md) |
| select! 中为什么使用 &mut Receiver 或 &mut JoinHandle？ | [保留接收端](libraries/tokio/oneshot.md#与-select-一起使用) · [临时借用与后续清理](libraries/tokio/select.md#阶段-12临时借用-joinhandle保留后续清理能力) |
| Some(result) 保存的是任务还是结果？句柄还在为什么不能再 await？ | [两种获胜路径](libraries/tokio/select.md#为什么保存-someresult另一个分支返回-none) |
| 超时能证明对端没有执行请求吗？ | [sleep 与 timeout：等待和时间上限](libraries/tokio/time.md) |
| abort 返回时，guard 一定已经销毁了吗？ | [取消：abort 之后为什么还要 await](libraries/tokio/cancellation.md) |
| 为什么要累积数据直到读到请求头结束标记？ | [异步网络：bind、accept 与分段读写](libraries/tokio/networking.md) |

## Axum

| 问题 | 阅读入口 |
|---|---|
| 注册路由时 chat 已执行了吗？ | [Router 与 Handler：注册和执行是两件事](libraries/axum/router-and-handler.md) |
| 框架根据参数名字还是参数类型选择提取规则？ | [State：框架怎样准备共享状态](libraries/axum/state.md) |
| 中间件怎样把请求 ID 交给处理函数？Extension 与 State 有什么区别？ | [Extension：请求内按类型传值](libraries/axum/extension.md) |
| 只定义 State 结构体是否就具备提取能力？ | [用最小模型理解 Extractor](libraries/axum/extractor-mechanics.md) |
| State、Json、Path 分别从哪里取数据？ | [Json 与 Path：从请求准备参数](libraries/axum/json-and-path.md) |
| 返回 Result 和返回 HTTP 500 是同一层概念吗？ | [响应转换：handler 可以返回什么](libraries/axum/responses.md) |

## Reqwest

| 问题 | 阅读入口 |
|---|---|
| Axum post 和 Reqwest post 分别做什么？ | [Client 与请求构建](libraries/reqwest/client-and-request.md) |
| 收到 503 为什么仍可能是 Ok(response)？ | [响应头、正文与错误分类](libraries/reqwest/response-and-errors.md) |
| 1 秒与 2 秒这两层超时各覆盖什么？ | [单次请求超时与总超时](libraries/reqwest/timeouts.md) |

## Serde 与 serde_json

| 问题 | 阅读入口 |
|---|---|
| 结构体字段公开与能否序列化是否是同一回事？ | [Serialize 与 Deserialize](libraries/serde/serialize-and-deserialize.md) |
| round-robin 怎样对应 RoundRobin？ | [字段与枚举的 JSON 映射](libraries/serde/field-and-enum-mapping.md) |
| 空 workers 能被解析，为什么仍然不能启动？ | [JSON 解析与业务校验](libraries/serde/json-parsing-and-validation.md) |

## url

| 问题 | 阅读入口 |
|---|---|
| 能解析的 URL 是否一定是合法 Worker 地址？ | [URL 解析与地址校验](libraries/url/parsing-and-validation.md) |

## 项目代码导读

| 问题 | 阅读入口 |
|---|---|
| 请求被取消时，谁负责计数减一？ | [代码导读：Worker 与在途请求的生命周期](walkthroughs/worker-lifecycle.md) |
| 每次新建 RoundRobin 为什么总从第一个开始？ | [代码导读：轮询选择与策略状态](walkthroughs/policy-selection.md) |
| 哪一层决定错误怎样打印，哪一层保留底层原因？ | [代码导读：配置加载与分层错误](walkthroughs/configuration-loading.md) |
| 为什么取得快照后才能释放注册表锁并等待网络？ | [代码导读：一次聊天请求](walkthroughs/chat-forwarding.md) |
| Err(response) 与上游 503 是否走相同的重试分支？ | [代码导读：重试与总期限](walkthroughs/retry-and-timeout.md) |
| Drop 的兜底和显式 cancel_and_join 分别保证什么？ | [代码导读：ManagedTask 的清理责任](walkthroughs/task-cleanup.md) |
| 为什么 Arc<()> 身份比只比较状态枚举更有用？ | [代码导读：许可、状态版本与迟到结果](walkthroughs/circuit-permit.md) |

## 进阶与扩展

| 问题 | 阅读入口 |
|---|---|
| 外层借用可以缩短，为何不能随意替换内层引用类型？ | [协变与不变：从 Java 泛型到 Rust 引用](advanced/variance.md) |
| fn(&str) 与某个固定 fn(&'a str) 能直接画等号吗？ | [高阶生命周期与函数输入逆变](advanced/higher-ranked-lifetimes.md) |
| 当前 guard 的借用设计与上游共享拥有设计有什么不同？ | [扩展：上游的 Arc 与 Weak 关系](advanced/upstream-ownership.md) |
| thiserror 自动生成能力，是否等于自动处理错误？ | [扩展：thiserror 的错误派生](advanced/thiserror.md) |
| 派生参数解析和业务校验有什么不同？ | [扩展：clap 的参数派生](advanced/clap.md) |
| 变型规则与编译器检查遗漏为什么是两回事？ | [历史案例：生命周期约束检查遗漏](advanced/compiler-case-study.md) |
| 什么情况下底层保留具体错误、高层再决定响应？ | [设计对照：责任、机制与策略](advanced/design-decisions.md) |
| 只读保存能否使用 &Payment，为什么？ | [扩展：支付对象的所有权选择](advanced/payment-ownership.md) |
| 禁止 Clone 是否能替代乐观锁？ | [扩展：Clone 与持久化一致性](advanced/payment-consistency.md) |
| 为什么 List<Cat> 不能作为 List<Animal>？ | [Java 泛型与 PECS：变型的入门对照](advanced/java-variance.md) |
| 为什么缩短借用时间不等于允许写入任意短引用？ | [两层引用：外层生命周期与内层类型](advanced/reference-variance.md) |
| 小值对象与可变业务实体应如何区分？ | [扩展：支付值对象与 Copy](advanced/payment-values.md) |

## 旧笔记中的其他提问

| 问题 | 阅读入口 |
|---|---|
| workspace、package、crate 分不清 | [Cargo Workspace、Package 与 Crate](concepts/project/workspace.md) |
| Cargo 如何识别 `src/bin/fake-worker.rs` | [Cargo 如何识别 library 和 binary](concepts/project/package-and-crate.md) |
| 为什么要这样设计错误和配置边界 | [Rust 软件设计的第一性原理](advanced/design-decisions.md) |
| Permit 与请求、熔断器是什么关系 | [显式保存请求与状态的关系](walkthroughs/circuit-permit.md) |
| generation 为什么不是 request_id | [状态版本与迟到结果](walkthroughs/circuit-permit.md) |
| Permit 与 SOLID、系统设计有什么关系 | [职责、版本校验与生命周期](advanced/design-decisions.md) |
| `pub mod`、`crate::`、`use` 看不懂 | [模块与可见性](concepts/project/package-and-crate.md) |
| `mod retry;` 为什么关联到 `gateway/retry.rs` | [模块声明如何定位文件](concepts/project/modules-and-files.md) |
| `circuit_breaker.rs` 与同名目录是两个模块吗，super 如何找到父类型 | [文件、目录与模块树的对应关系](concepts/project/modules-and-files.md) |
| `gateway/tests.rs` 和 `gateway/tests/mod.rs` 有什么区别 | [两种模块文件布局](concepts/project/modules-and-files.md) |
| `type HttpResponse` 是新类型吗 | [类型别名](concepts/types/aliases-and-newtypes.md) |
| `Future<Output = ...>` 中 Output 是泛型吗 | [关联类型 Output](concepts/types/associated-types.md) |
| 泛型参数与关联类型有什么区别 | [泛型与关联类型对照](concepts/types/associated-types.md) |
| 协变、逆变、不变和 Java extends / super 怎么对应 | [从 Java 泛型理解 Rust 变型](advanced/variance.md) |
| `&&str`、`*slot` 与内外两层生命周期看不懂 | [两层引用与解引用](advanced/reference-variance.md) |
| 为什么 `&mut T` 对 T 不变，却能缩短借用时间 | [外层生命周期与内层类型](advanced/reference-variance.md) |
| `Vec<T>` 协变，为什么不能自动转成 trait 对象列表 | [trait 约束与 Java 类继承](advanced/variance.md) |
| 为什么 timeout 得到 Ok(Err(...)) | [重试与两层 Result](concepts/types/aliases-and-newtypes.md) |
| `use of moved value` | [所有权、转移与借用](concepts/ownership/move-and-partial-move.md) |
| `&T` 与 `&mut T` 不清楚 | [所有权、转移与借用](concepts/ownership/move-and-partial-move.md) |
| `url.as_str()` 与 `&url` 有什么区别 | [字符串借用与 IntoUrl](concepts/values/string-and-str.md) |
| post 返回 MethodRouter，为什么里面写闭包 | [Axum：语法层次与类型约束](libraries/axum/router-and-handler.md) |
| async move 是不是闭包，Handler 为什么返回 Future | [Axum：post、Handler 与 Future](libraries/axum/router-and-handler.md) |
| `mut self` 是不是 `&mut self`，take 是不是 clone | [消费自身与 Option::take](walkthroughs/task-cleanup.md) |
| `InFlightGuard<'a>` 为什么不能比 Worker 活得更久 | [guard 借用 Worker 与生命周期检查](concepts/ownership/lifetimes.md) |
| `&[Worker]` 是什么 | [借用切片与 Vec 类型标注](concepts/values/arrays-slices-and-vec.md) |
| 空 `Vec` 无法推断类型 | [借用切片与 Vec 类型标注](concepts/values/arrays-slices-and-vec.md) |
| turbofish 干什么，常见场景有哪些 | [turbofish：给泛型指定类型](concepts/types/type-inference.md) |
| `parse::<u64>()` 看不懂 | [`parse::<u64>()`：给泛型方法指定目标类型](concepts/values/arrays-slices-and-vec.md) |
| `pending::<()>()` 中两个 `()` 什么意思 | [pending：两个括号不同](concepts/async/future-and-await.md) |
| `State(state): State<AppState>` 看不懂 | [`State(state)`：解构元组结构体](concepts/patterns/destructuring.md) |
| extractor 是什么，框架怎样根据参数类型准备值 | [Extractor 最小例子](libraries/axum/extractor-mechanics.md) |
| State 是谁定义的，为什么包装后就能提取状态 | [State 与提取规则](libraries/axum/extractor-mechanics.md) |
| `dispatch(&state, health)` 怎样推断出 State<AppState> | [泛型推断与 trait 实现选择](libraries/axum/extractor-mechanics.md) |
| `let Some(index) = value else` 看不懂 | [`let ... else`：取出值，失败就提前离开](concepts/patterns/if-let-and-let-else.md) |
| `&config.policy` 借用哪个值 | [字段借用与 `match` 表达式](concepts/values/expressions-and-semicolons.md) |
| `match` 后的分号为什么改变返回值 | [字段借用与 `match` 表达式](concepts/values/expressions-and-semicolons.md) |
| `Err(error) if error.is_cancelled()` 怎么匹配 | [match 分支守卫](concepts/patterns/match-and-guards.md) |
| `Some(ref value)` 或锁守卫解引用 | [`if let`、`ref` 与解引用](concepts/patterns/if-let-and-let-else.md) |
| `?` 为什么会提前返回 | [`Result`、`?` 与错误传播](concepts/errors/question-mark.md) |
| `?` 能用于 Option 吗，能和 Result 混用吗 | [Option 与 Result 的传播和转换](concepts/errors/question-mark.md) |
| `handle.await??` 为什么有两个问号 | [逐层处理嵌套结果](concepts/errors/nested-results.md) |
| `.into()` 如何推断目标，From 是不是反向转换 | [Into、From 与可失败转换](concepts/errors/from-and-into.md) |
| `try_into()` 与 `into()` 有什么区别 | [可能失败的转换](concepts/errors/fallible-conversions.md) |
| `Result`、`unwrap`、`expect` 如何选择 | [`Result`、`?` 与错误传播](concepts/errors/question-mark.md) |
| `Formatter<'_>` 和 `'_` 是什么 | [Formatter、生命周期、Display 与 Error](concepts/errors/debug-display-and-formatter.md) |
| `Display` 与 `Debug` 有什么区别 | [Formatter、生命周期、Display 与 Error](concepts/errors/debug-display-and-formatter.md) |
| 实现 `Error` 到底有什么用 | [Formatter、生命周期、Display 与 Error](concepts/errors/debug-display-and-formatter.md) |
| `Error`、`From`、`?` 如何配合 | [Formatter、生命周期、Display 与 Error](concepts/errors/debug-display-and-formatter.md) |
| `From` 与 `source()` 有什么区别 | [From 与 source 对照](concepts/errors/error-and-source.md) |
| 不知道该派生哪些 trait | [常见 derive 速查](concepts/types/derive-and-default.md) |
| `P: Trait` 与 `dyn Trait` 的区别 | [Trait 与分发](concepts/types/traits-and-bounds.md) |
| `ManagedTask<T>` 和 `impl<T>` 中的 T 是什么 | [泛型结构体与实现](concepts/types/generics-and-inference.md) |
| `\|\|`、`impl FnMut() -> bool` 怎么读 | [闭包与 Fn 系列](concepts/types/closures-and-fn.md) |
| 外层 move 和内层 async move 为什么能重复调用 | [双层捕获与 Copy](concepts/async/async-capture.md) |
| 为什么需要 `Box<dyn Trait>` | [Trait 与分发](concepts/types/traits-and-bounds.md) |
| `Box<dyn Policy + Send>` 为什么放进 `Mutex` | [配置驱动的共享策略](walkthroughs/policy-selection.md) |
| 每次 `RoundRobin::new()` 为什么只选第一个 | [配置驱动的共享策略](walkthroughs/policy-selection.md) |
| `Arc` 为什么不能直接修改内部值 | [Trait 与分发](concepts/types/traits-and-bounds.md) |
| `Arc` 循环引用或 `Weak::upgrade()` | [`Arc`、`Weak` 与循环引用](concepts/sharing/weak-and-cycles.md) |
| `Arc::ptr_eq` 与值相等有什么区别 | [对象身份与值相等](walkthroughs/circuit-permit.md) |
| 节点已经移除，Arc 保住的到底是什么 | [节点移除与对象销毁](walkthroughs/worker-lifecycle.md) |
| 从运行时视角理解关注点分离 | [学习感悟 realization](walkthroughs/worker-lifecycle.md) |
| `MutexGuard<'_, T>` 是什么 | [`Mutex`、`MutexGuard`、解引用与毒锁](concepts/sharing/mutex-and-guards.md) |
| 为什么要写 `*guard` | [MutexGuard 为什么能解引用](concepts/sharing/deref-and-guards.md) |
| `lock()` 为什么返回 `Result` | [`lock()` 与毒锁](concepts/sharing/poisoning.md) |
| 为什么不能拿着锁 `.await` | [Guard 与 `.await`](concepts/sharing/mutex-and-guards.md) |
| `AtomicUsize` 为什么能通过 `&self` 加一 | [`AtomicUsize` 与内部可变性](concepts/sharing/atomics.md) |
| `Ordering::Relaxed` 是不是不再原子 | [`Relaxed` 到底放松了什么](concepts/sharing/atomics.md) |
| Runtime、Future、async/await、任务与线程分不清 | [Tokio 基础专题](libraries/tokio/runtime-task-thread.md) |
| `async move` 能否省略，什么时候需要 move | [捕获推断与 move](concepts/async/async-capture.md) |
| Future 暂停后，变量和 guard 为什么还在 | [Future 保存执行状态](concepts/async/async-capture.md) |
| move 为什么不保证 Send，static 是否表示永远存活 | [Send 与 static 的边界](libraries/tokio/spawn.md) |
| JoinHandle 来自标准库还是 Tokio，像不像 Runnable | [JoinHandle：不是 Runnable](libraries/tokio/join-handle.md) |
| timeout 中检查计数的循环是什么意思 | [sleep、timeout 和 pending](libraries/tokio/time.md) |
| abort 后为什么还要 await，guard 为什么会 Drop | [取消测试完整过程](libraries/tokio/cancellation.md) |
| TestServer 没有实现 Drop，为什么仍会请求取消 | [组合对象与任务资源管理](walkthroughs/task-cleanup.md) |
| 测试该抽哪些辅助逻辑，断言放在哪里 | [测试重构的设计原则](walkthroughs/task-cleanup.md) |
