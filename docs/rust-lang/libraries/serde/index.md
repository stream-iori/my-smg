# Serde 与 serde_json

[总入口](../../index.md) · [问题索引](../../questions.md)

以当前 Cargo.lock 锁定的 **serde 1.0.229 / serde_json 1.0.154** 为例。Manifest 的版本范围和最终锁定版本要分开看。

[对应版本官方文档](https://docs.rs/serde/1.0.229/serde/)。按[当前阶段](../../stage-reading.md)选择文章；目录用于回查。前置只要求对应基础部分，项目应用另补该节前置。

| 文章 | 解决的问题 | 前置 | 源码入口 |
|---|---|---|---|
| [Serialize 与 Deserialize](serialize-and-deserialize.md) | 结构体字段公开与能否序列化是否是同一回事？ | [struct、enum、impl 与 Self](../../concepts/values/struct-enum-and-impl.md) · [derive 与 Default：编译器替你生成什么](../../concepts/types/derive-and-default.md) | [src/gateway.rs](../../../../src/gateway.rs)：`#[derive(Deserialize, Serialize)]` |
| [字段与枚举的 JSON 映射](field-and-enum-mapping.md) | round-robin 怎样对应 RoundRobin？ | [Serialize 与 Deserialize](serialize-and-deserialize.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../../src/config.rs)：`#[serde(rename_all = "kebab-case")]` |
| [JSON 解析与业务校验](json-parsing-and-validation.md) | 空 workers 能被解析，为什么仍然不能启动？ | [Serialize 与 Deserialize](serialize-and-deserialize.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 | [src/config.rs](../../../../src/config.rs)：`from_json_validated` |
