# 代码导读：一次聊天请求

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**请求先获得节点快照，再选择节点、保持在途 guard、申请尝试许可并转发响应。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [State：框架怎样准备共享状态](../libraries/axum/state.md) · [Client 与请求构建](../libraries/reqwest/client-and-request.md) · [Mutex 与 Guard：锁何时释放](../concepts/sharing/mutex-and-guards.md) |
| 源码入口 | [src/gateway.rs](../../../src/gateway.rs)：`chat` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 一次请求的主线

```text
State / Json 提取参数
    ↓
锁住注册表 → snapshot → 解锁
    ↓
锁住策略 → select → 解锁
    ↓
没有健康节点？→ 返回 503
    ↓ 有节点
创建在途 guard（覆盖整次逻辑请求）
    ↓
创建可重复调用的 operation
    ↓ 每次尝试
申请 CircuitPermit → send → 读正文 → 上报结果
    ↓
重试次数/退避 + 外层总期限
    ↓
返回状态和正文，作用域清理在途 guard
```

| 对象 | 保存在哪里 | 什么时候释放 |
|---|---|---|
| 注册表/策略守卫 | 短代码块 | 网络 await 之前 |
| 节点 Arc | 请求快照 | 请求 Future 清理时 |
| 在途 guard | chat 的局部状态 | 整次逻辑请求结束时 |
| CircuitPermit | 每次 operation 的 Future | 本次尝试上报或取消清理时 |

当前源代码中变量拼写是 selected_idex，解释中称“选中索引”。文档不借重组修正学习者源码。
operation 捕获的是 client、url_ref、request_ref、gate 等引用；move 不表示把整份客户端或请求一定搬走。

项目没有自动调用 error_for_status，因此完整上游 503 可以作为 Ok(response) 进入状态码重试判断；发送或正文错误映射成 Err 后立即结束通用重试。

这是当前实现的读图，不声称已支持 SSE 或优雅停机；那些仍属于后续课程阶段。

## 自查与关联

为什么取得快照后才能释放注册表锁并等待网络？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：配置加载与分层错误](configuration-loading.md)
