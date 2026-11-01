# 字段与枚举的 JSON 映射

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Serde 属性决定外部名字怎样对应 Rust 字段或变体，不改变 Rust 模块路径。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Serialize 与 Deserialize](serialize-and-deserialize.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`#[serde(rename_all = "kebab-case")]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 项目把配置名映射到 Rust 枚举

摘自 `src/config.rs`：

```rust
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyKind {
    FirstHealthy,
    RoundRobin,
}
```

| JSON 配置值 | Rust 变体 |
|---|---|
| `"first-healthy"` | PolicyKind::FirstHealthy |
| `"round-robin"` | PolicyKind::RoundRobin |

`rename_all` 改变外部表示，不会把 Rust 源码里的变体改名，也不会改变模块路径。
当前 WorkerConfig/GatewayConfig 的字段按默认名字对应 JSON，字段类型决定如何解析。

| 属性/机制 | 是否在当前配置使用 | 边界 |
|---|---|---|
| rename_all | 是 | 统一变体外部名字 |
| rename | 否，扩展对照 | 单独改一个字段或变体的外部名字 |
| default | 否，扩展对照 | 允许缺失字段由指定默认规则补齐 |

普通必填字段缺失会报错；未知字段默认行为不同于缺失字段，不能把“解析成功”当成严格检查每个业务条件。
参考 [Serde 属性](https://serde.rs/attributes.html)、[枚举属性](https://serde.rs/container-attrs.html)。

## 自查与关联

round-robin 怎样对应 RoundRobin？

[Serialize 与 Deserialize](serialize-and-deserialize.md) · [JSON 解析与业务校验](json-parsing-and-validation.md)
