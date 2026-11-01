# 响应头、正文与错误分类

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**send 成功不等于状态码成功，也不等于正文已完整读完。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`upstream.status() / upstream.text().await` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 两个等待点分别可能失败

```text
send().await
├── Err：连接、发送、等待响应等失败
└── Ok(Response)：取得响应头和正文读取入口
         ├── status()：读取状态码，不消费正文
         └── text().await
              ├── Err：正文读取/解码过程失败
              └── Ok(String)：完整文本
```

当前 chat 会分别调用 `upstream_error_response(&error, "request")` 和 `... "response"`，保留出错阶段。
`error.is_timeout()` 映射为 504，其余请求/正文错误映射为 502。

| 情况 | 能否仅凭 send 的 Ok 认定成功 |
|---|---|
| 上游返回 503 | 不能；没有调用 error_for_status 时，Response 本身仍可成功取得 |
| 200 头已到，正文随后断开或超时 | 不能；text().await 仍可能失败 |
| 完整响应取得 | 网络读取完成；业务成功还要由状态码和业务规则判断 |

项目读完正文后才上报 permit 成功或失败，避免把只有响应头的请求误判成完整完成。
参考 [Response](https://docs.rs/reqwest/0.13.5/reqwest/struct.Response.html)、[Error](https://docs.rs/reqwest/0.13.5/reqwest/struct.Error.html)。

## 自查与关联

收到 503 为什么仍可能是 Ok(response)？

[Client 与请求构建](client-and-request.md) · [单次请求超时与总超时](timeouts.md)
