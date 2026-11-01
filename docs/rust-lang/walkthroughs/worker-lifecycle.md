# 代码导读：Worker 与在途请求的生命周期

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**从拥有节点到创建 guard，再到释放计数，沿对象存活而不是只看函数调用理解流程。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Rc 与 Arc：多个句柄共同拥有数据](../concepts/sharing/rc-and-arc.md) · [Drop 与 RAII：离开作用域如何收尾](../concepts/ownership/drop-and-raii.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker.rs](../../../src/worker.rs)：`Worker / InFlightGuard / OwnedInFlightGuard` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 当前非流式 chat 的对象关系

```text
WorkerRegistry ──Arc──┐
                     ├──> Worker
请求局部 snapshot ───┘       ↑
                       InFlightGuard 借用
```

当前快照类型为 Vec<Arc<Worker>>。非流式 `chat` 仍使用字段为 `&Worker` 的借用型 guard，它没有自己拥有 Arc；是请求局部快照中的 Arc 保证 Worker 存活。
阶段 11 另有拥有型 guard，见下文；类型已经存在不代表流式正文已接入它。

| 时机 | 状态变化 |
|---|---|
| registry.snapshot | 克隆 Arc，得到独立的快照容器，不深拷贝节点 |
| policy.select | 返回快照中的索引或 None |
| begin_request | 原子计数加一，返回借用 Worker 的 guard |
| 从注册表移除 | 后续新快照不包含它，已有快照仍可能持有它 |
| 请求返回、超时或取消清理 | guard Drop 减一；局部快照释放 Arc |

这里“移除后不再被新快照选到”不等于原有快照会被自动撤回；当前 remove 不会自动取消已经选中的请求。
健康检查通过 AtomicBool 更新对象状态，快照中的 Arc 指向同一对象，所以能观察后续健康变化。

## 节点移除与对象销毁是两件事

从注册表移除节点只是释放注册表持有的 Arc，不会直接销毁所有调用者拥有的 Worker。

```text
移除前：注册表 ──Arc──┐
                     ├──> Worker ← 借用 ── 在途 guard
        请求快照 ────┘
移除后：请求快照 ──Arc──> Worker 仍存活
请求结束：guard 减计数，快照释放；最后一个 Arc 释放时 Worker 才销毁
```

当前非流式 `chat` 的 guard 保存 &Worker，快照保存 Arc<Worker>；这条既有调用路径要与新增的拥有型 guard 分开分析。

## 如果 guard 只保存引用

| guard 保存什么 | 能否独立保住对象 | 当前 Vec 场景下移除会怎样 |
|---|---|---|
| `&Worker` | 不能；引用不是拥有者 | 移除破坏仍有效的借用时，编译器拒绝代码 |
| `Arc<Worker>` | 能；guard 是共同拥有者 | 可从列表移除，对象由剩余 Arc 保持存活 |

对于早期同步版本或独立示例中的 `Vec<Worker>`，下面的顺序不能编译：

```rust,compile_fail
let guard = workers[0].begin_request();
workers.remove(0); // 需要可变访问列表，但 guard 仍借用其中的节点
drop(guard);
```

“编译器保证”不是运行时自动等待：它在编译时拒绝危险顺序。先销毁 guard，再移除节点，
才符合这个借用设计。并非任何引用都禁止注册表修改；如果先独立持有 Arc，再从它借用，
对象存活的根本保证仍来自那份 Arc。

## 节点删除：分开可选资格与对象存活

这次对照 `my-smg` 和 SGL Model Gateway 得到的感悟是：

> 停止接新活，不破坏已经开始的活。

不要把“移除节点”看成一个不可拆的动作。从运行时需求看，它包含两个独立问题：

| 关注点 | 要回答什么 | 负责的机制 |
|---|---|---|
| 可选资格 | 后续请求还能不能选到节点 | 注册表、健康状态 |
| 对象存活 | 已有请求能否继续访问节点并完成收尾 | 请求、guard 持有的 Arc |

```text
从注册表移除 → 新查询不再得到该条目
已有请求保留 Arc → Worker 仍存活，直到最后一个拥有者释放
```

所以“业务上不再可用”不等于“内存中立即销毁”。注册表的变化不需要迫使已有请求提前结束。
这里的对象是网关内的 Worker 状态，不是远端服务进程。

## 引用与 Arc 的选择来自需求，不来自语法喜好

引用不会增加所有者；编译器检查它是否一直有效。当前借用 Vec 元素的 guard 尚未释放时，
会阻止破坏借用的移除操作，而不是自动等请求结束再执行移除。

Arc 让请求成为独立的共同拥有者，因此允许注册表删除自己的条目，又不销毁请求正在使用的对象。
两种方案都可以安全，但对“删除什么时候发生”的约束不同。

## 阶段 11：拥有型 guard 独立保存节点句柄

新增 `OwnedInFlightGuard` 的目标是：让将来的流式正文持有计数凭证，不依赖 handler 的局部快照继续存活。
目前 [Worker](../../../src/worker.rs) 已有创建方法和 guard 类型；[网关 chat_stream](../../../src/gateway.rs) 尚未把这个 guard 交给正文。

```text
worker：Arc<Worker>
    |
    | begin_request_owned() 自动借用接收者
    v
self：&Arc<Worker>
    +-- 在途请求计数 +1
    +-- Arc::clone(self)
              |
              v
       OwnedInFlightGuard
              +-- Arc<Worker> ----共同拥有----> Worker
                                                 ^
调用方原来的 Arc<Worker> -------------------------+

guard 可以移出创建它的作用域
    |
    v
销毁 guard --> 在途请求计数 -1 --> 自动销毁内部 Arc 字段
```

### 两套计数，分别回答两个问题

| 计数 | 回答什么 | 哪个操作改变它 |
|---|---|---|
| Arc 强引用计数 | 还有多少共享拥有句柄维持 Worker 存活？ | 创建、克隆和销毁 Arc 句柄 |
| Worker 在途请求计数 | 有多少请求尚未结束？ | 创建请求 guard 时加一，销毁 guard 时减一 |

单独克隆一个观察用 Arc 不创建业务请求，所以在途数不变。
移动 guard 转移清理责任，不额外增加请求数，也不会在移动点减计数。

| 需要解释的知识 | 主讲解位置 |
|---|---|
| Arc::new 为什么得到 Arc，而不是 &Arc | [句柄与引用的类型表](../concepts/sharing/rc-and-arc.md#arctarct-与-t-要分开) |
| worker 可以直接调用接收 &Arc 的方法 | [显式接收者与自动借用](../concepts/ownership/method-receivers.md#阶段-11self-与-self-arcself) |
| 借用型 guard 为什么带 'a，Drop 为什么也带 | [两种 guard 的生命周期](../concepts/ownership/lifetimes.md#阶段-11借用型-guard-与拥有型-guard) |
| Drop 后 Arc 怎样释放 | [自定义 Drop 与字段清理](../concepts/ownership/drop-and-raii.md#阶段-11自定义-drop-之后字段仍会自动销毁) |

验收这个资源模型时，应分别观察：创建后计数为 1、移出原作用域后仍有效、移动 guard 不减计数、销毁后归零。
这些条件通过后，才继续验证正文持有、错误和取消路径，不能把 guard 的单元行为当成整个 SSE 资源生命周期已验收。

## 自查与关联

请求被取消时，谁负责计数减一？

[代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：配置加载与分层错误](configuration-loading.md) · [代码导读：一次聊天请求](chat-forwarding.md)
