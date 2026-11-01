# URL 解析与地址校验

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Url 解析结构，业务代码再限制协议和主机等条件；String 只是文本。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`GatewayConfig::validate` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 文本地址与结构化 URL

项目首先调用 `Url::parse(worker.address.as_str())`，再检查 `scheme()` 是否为 http 或 https。

```text
String：一段地址文本
    ↓ Url::parse
Result<Url, ParseError>
    ↓ 解析成功
读取 scheme 等结构字段
    ↓ 业务允许的协议？
接受 / ConfigError::InvalidWorkerAddress
```

| 输入示意 | 解析/校验结果 |
|---|---|
| `http://127.0.0.1:3001` | 可以解析，协议符合当前规则 |
| `not-a-url` | 无基准的普通相对文本，parse 失败 |
| `ftp://example.com` | 可以解析，但当前 Worker 规则拒绝 ftp |

当前 validate 检查解析与协议，没有额外的独立 host/路径/用户信息白名单规则。不要把建议增加的检查写成已实现行为。
`url` 是库名，`Url` 是类型；项目里名叫 url 的变量也可能只是 String，必须看声明。
参考 [url 2.5.8 Url](https://docs.rs/url/2.5.8/url/struct.Url.html)。

## 自查与关联

能解析的 URL 是否一定是合法 Worker 地址？
