# url

[总入口](../../index.md) · [问题索引](../../questions.md)

以当前 Cargo.lock 锁定的 **2.5.8** 为例。Manifest 的版本范围和最终锁定版本要分开看。

[对应版本官方文档](https://docs.rs/url/2.5.8/url/)。按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [URL 解析与地址校验](parsing-and-validation.md) | 能解析的 URL 是否一定是合法 Worker 地址？ | [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md) | [src/config.rs](../../../../src/config.rs)：`GatewayConfig::validate` |
