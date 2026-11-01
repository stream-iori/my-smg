# 按当前任务阅读 Rust 文档

[总入口](index.md) · [Java 迁移地图](java-to-rust.md) · [课程与验收](../../learning/index.md)

**先确认任务，再读最少概念；自己实现并验收后，才读完整调用链。** 此页不改变课程阶段，也不代表学习者已经通过验收。

```text
当前任务 → 必读基础部分 → 自己实现 → 行为验收与解释
                ↑                         ↓
          卡住时按需回查              验收后项目导读
```

“基础部分”指开头到“基础自查”，包括该自查；明确标注后续阶段的内容可以跳过。前置只要求对应基础部分，不递归通读全文。源码入口供定位，不要求把当前完整实现搬回早期任务。

## 阶段 0～2：T00～T07

目标、样例、验收以[首批任务](../../learning/06-first-tasks.md)为准，下表每一行只在做到对应任务时使用。

| 任务 | 开始前必读 | 遇到问题再查 | 验收后阅读 |
|---|---|---|---|
| T00：工程 | [Package/Crate](concepts/project/package-and-crate.md)、[模块](concepts/project/modules-and-files.md)、[路径/use](concepts/project/paths-and-imports.md)基础部分 | [可见性](concepts/project/visibility.md)、[edition](concepts/project/edition.md) | 复述库与二进制边界，暂不读业务导读 |
| T01：节点 | [struct/enum/impl](concepts/values/struct-enum-and-impl.md)、[String/str](concepts/values/string-and-str.md) | [Move](concepts/ownership/move-and-partial-move.md)基础部分 | 画出节点和文本的拥有关系 |
| T02：状态修改 | [借用](concepts/ownership/borrowing.md)、[接收者](concepts/ownership/method-receivers.md)基础部分；有疑问先补 Move | [可变绑定的 Java 对照](java-to-rust.md) | 解释状态变化与修改权限 |
| T03：筛选 | [数组/切片/Vec](concepts/values/arrays-slices-and-vec.md)、[迭代](concepts/values/iteration-and-collection.md)基础部分 | [空 Vec 类型推断](concepts/types/type-inference.md)基础部分 | 对照索引与原集合位置，暂不读注册表 |
| T04：轮询 | [表达式/分号](concepts/values/expressions-and-semicolons.md)、[Option](concepts/errors/option-and-result.md)、[match](concepts/patterns/match-and-guards.md)基础部分 | [if let/let-else](concepts/patterns/if-let-and-let-else.md)基础部分 | 解释计数器归谁，每次调用改变什么 |
| T05：健康变化 | 复用 T02～T04，不新增概念 | 回查借用、索引、无值分支 | 验证下线与恢复，不提前引入并发 |
| T06：解释借用 | [Move](concepts/ownership/move-and-partial-move.md)、[借用](concepts/ownership/borrowing.md)、[Copy/Clone](concepts/ownership/copy-and-clone.md)基础部分 | [生命周期](concepts/ownership/lifetimes.md)基础部分 | 不看源码解释、重写并验收 |
| T07：记录 | 按[学习流程](../../learning/05-workflow.md)记录，不新增阅读 | 回查未解释清楚的问题 | 按[源码对照指南](../../learning/04-source-map.md)读阶段 2 对应上游文件，然后进入阶段 3 |

这条路径仅需标准库同步知识；trait、Box、Arc、原子变量、Tokio、Axum、在途 guard 均不是阶段 0～2 的实现要求。

## 阶段 3～9

按[主线课程](../../learning/02-mainline.md)推进；文章内明确标注更后阶段的应用仍延后阅读。

| 阶段 | 开始前必读 | 遇到问题再查 | 验收后阅读 |
|---|---|---|---|
| 3：策略抽象 | [泛型](concepts/types/generics-and-inference.md)、[trait](concepts/types/traits-and-bounds.md)、[Box](concepts/sharing/box.md)基础部分、[分发](concepts/types/static-and-dynamic-dispatch.md) | [推断](concepts/types/type-inference.md)、[dyn 条件](concepts/types/dyn-compatibility.md)、[derive](concepts/types/derive-and-default.md)基础部分 | [策略导读](walkthroughs/policy-selection.md)同步对照；共享部分延至阶段 7 |
| 4：配置与错误 | [Result](concepts/errors/option-and-result.md)、[From/Into](concepts/errors/from-and-into.md)、[问号](concepts/errors/question-mark.md)、[JSON 校验](libraries/serde/json-parsing-and-validation.md)、[URL 校验](libraries/url/parsing-and-validation.md) | [错误链](concepts/errors/error-and-source.md)、[Display](concepts/errors/debug-display-and-formatter.md)、[Default](concepts/types/derive-and-default.md)、[字段映射](libraries/serde/field-and-enum-mapping.md) | [配置导读](walkthroughs/configuration-loading.md) |
| 5：假服务 | [Future](concepts/async/future-and-await.md)、[Runtime](libraries/tokio/runtime-task-thread.md)基础部分、[路由](libraries/axum/router-and-handler.md)、[JSON 参数](libraries/axum/json-and-path.md) | [解构](concepts/patterns/destructuring.md)、[序列化](libraries/serde/serialize-and-deserialize.md)、[等待](libraries/tokio/time.md) | 对照自己的假服务入口；共享计数示例延至阶段 7 |
| 6：转发 | [Client](libraries/reqwest/client-and-request.md)、[响应/错误](libraries/reqwest/response-and-errors.md)、[State](libraries/axum/state.md)、[响应转换](libraries/axum/responses.md) | [Extractor](libraries/axum/extractor-mechanics.md)、[文本借用](concepts/values/string-and-str.md) | 画自己的请求数据流；当前完整聊天导读延至阶段 10 |
| 7：并发计数 | [Arc](concepts/sharing/rc-and-arc.md)、[Mutex](concepts/sharing/mutex-and-guards.md)、[原子变量](concepts/sharing/atomics.md)、[Send/Sync](concepts/sharing/send-and-sync.md)、[Drop](concepts/ownership/drop-and-raii.md) | [生命周期 guard](concepts/ownership/lifetimes.md)、[毒锁](concepts/sharing/poisoning.md)、[取消](libraries/tokio/cancellation.md) | [节点生命周期](walkthroughs/worker-lifecycle.md)，动态移除部分延至阶段 8 |
| 8：管理与探测 | [spawn](libraries/tokio/spawn.md)、[JoinHandle](libraries/tokio/join-handle.md)、[容器借用/取值](concepts/errors/option-access-and-take.md) | [快照迭代](concepts/values/iteration-and-collection.md)、[相等/哈希](concepts/types/comparison-and-hash.md)、[取消](libraries/tokio/cancellation.md) | [任务清理](walkthroughs/task-cleanup.md)、[节点移除](walkthroughs/worker-lifecycle.md) |
| 9：超时重试 | [闭包](concepts/types/closures-and-fn.md)、[捕获](concepts/async/async-capture.md)、[Instant/Duration](concepts/values/instant-and-duration.md)基础部分、[sleep/timeout](libraries/tokio/time.md)、[两层超时](libraries/reqwest/timeouts.md) | [关联类型](concepts/types/associated-types.md)、[嵌套结果](concepts/errors/nested-results.md) | [重试导读](walkthroughs/retry-and-timeout.md) |

阶段 5～6 的库文章含当前共享状态或完整重试实现时，只读本阶段接口职责；不能以“源码已经使用”为理由提前增加依赖或抽象。

## 阶段 10～12：已有材料与缺口

| 阶段 | 已有材料 | 尚待补充 | 验收依据 |
|---|---|---|---|
| 10：熔断/限流/排队 | [CircuitPermit](walkthroughs/circuit-permit.md)、[完整聊天路径](walkthroughs/chat-forwarding.md)、[Instant/Duration 与令牌桶时间计算](concepts/values/instant-and-duration.md) | 令牌桶接入与并发、Semaphore、有界 channel、等待队列 | 主线阶段 10 分别验收三项行为 |
| 11：SSE | [SSE 调度与所有权](walkthroughs/sse-scheduling.md)、[内存控制与背压](advanced/streaming-memory.md)；[取消](libraries/tokio/cancellation.md)、[资源释放](walkthroughs/worker-lifecycle.md)可回查 | 网关逐段转发、Bytes/Pin 的具体使用、正文持有 guard，以及断开与慢读的实现和验收 | 主线阶段 11；已有教学材料和假节点不代表网关已完成验收 |
| 12：观测/停机 | [任务清理](walkthroughs/task-cleanup.md)、[select!](libraries/tokio/select.md)、[oneshot](libraries/tokio/oneshot.md)、[Ordering](concepts/sharing/atomics.md)可回查 | tracing 与指标导读、关闭信号和限时排空的完整验收说明 | 主线阶段 12；abort 一个任务不是完整优雅停机 |

缺口随对应任务补充；已有概念材料不代表阶段实现或验收已经完成。上游对照始终在独立实现和验收之后。
