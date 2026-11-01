# 代码导读：重试与总期限

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**闭包是创建新 Future 的办法；业务重试与总超时分别管理次数和耗时。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [关联类型：实现确定自己的 Output](../concepts/types/associated-types.md) · [闭包与 Fn、FnMut、FnOnce](../concepts/types/closures-and-fn.md) · [sleep 与 timeout：等待和时间上限](../libraries/tokio/time.md) |
| 源码入口 | [src/gateway/retry.rs](../../../src/gateway/retry.rs)：`retry_with_backoff` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先看重试函数的四个角色

| 名称 | 角色 |
|---|---|
| F | 可重复调用的操作闭包类型 |
| Fut | 每次闭包调用后返回的具体 Future 类型 |
| Output | 该 Future 完成后的 Result 类型 |
| HttpResponse | (StatusCode, String) 的别名 |

```text
operation()
   ↓ 新的 Future
.await
   ├── Err(response) → ? 提前返回，不按状态码重试
   └── Ok((status, body))
          ├── 非 502/503/504，或达到三次上限 → 返回
          └── sleep(100ms × attempt) → 再调用 operation()
```

次数上限是 3 次尝试，不是首次之外再重试 3 次。每次上游尝试申请新的许可，但在途 guard 覆盖整次逻辑请求。
最外层 2 秒 timeout 把网络、正文与退避一起包住；不会无限延长。

## 同一个别名可以出现在 Ok 和 Err 中

```rust
Result<HttpResponse, HttpResponse>
```

等同于：

```rust
Result<(StatusCode, String), (StatusCode, String)>
```

| 分支 | 当前重试练习中的含义 |
|---|---|
| `Ok(response)` | 收到完整上游响应，允许继续判断状态码 |
| `Err(response)` | 发送或读取失败，已经转换成网关错误响应，直接结束 |

`Ok` 不代表状态码一定是 200：真实上游返回 503 仍可放在 Ok 中。
具体类型相同，不等于分支语义相同。

最初使用泛型 E 时，E 也不是 Error 的缩写约束：除非声明 `E: Error`，
否则它可以是任意满足当前接口的类型。本例 Err 内的值是响应元组，
编译器因此可以推断 E 为该元组，而不是原始的 reqwest::Error。

错误控制流见 [Result、? 与错误传播](../concepts/errors/question-mark.md)。

## 拆解当前重试函数的约束

```rust
F: FnMut() -> Fut,
Fut: Future<Output = Result<HttpResponse, HttpResponse>>,
```

| 名称 | 是什么 |
|---|---|
| F | 闭包的具体类型，函数的泛型参数 |
| Fut | 闭包调用后产生的 Future 类型，函数的泛型参数 |
| Output | Future trait 中定义的关联类型名称，不是额外声明的泛型参数 |
| HttpResponse | 元组的类型别名 |

```text
operation：F
    ↓ operation()：调用闭包
Future：Fut
    ↓ .await：取得 Output
Result<HttpResponse, HttpResponse>
    ↓ ?
Ok：取出元组继续判断状态；Err：从重试函数提前返回
```

注意：这里的 Output 是整个 Result，不只是其中的 HttpResponse。
闭包是可重复调用的创建办法，每次创建新的 Future；完成过的 Future 不是可重放的任务模板。
关联阅读：[闭包与 Fn 系列](../concepts/types/closures-and-fn.md)。

## 自查与关联

Err(response) 与上游 503 是否走相同的重试分支？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：配置加载与分层错误](configuration-loading.md)
