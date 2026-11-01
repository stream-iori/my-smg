# JSON 解析与业务校验

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**解析检查数据能否构造目标类型，业务校验再判断配置是否允许。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Serialize 与 Deserialize](serialize-and-deserialize.md) · [Option 与 Result：有无与成败](../../concepts/errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`from_json_validated` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 项目已经把两步拆开

```rust
pub fn from_json_validated(input: &str) -> Result<Self, ConfigLoadError> {
    let config = Self::from_json(input)?;
    config.validate()?;
    Ok(config)
}
```

这是 `impl GatewayConfig` 内的源码摘录。

```text
JSON 文本 → serde_json::from_str → GatewayConfig → validate → 可用配置
              检查类型结构                       检查业务规则
```

空 workers 数组符合 Vec 的类型要求，但项目不能用空节点集启动，因此 validate 返回 EmptyWorkers。
重复节点 ID、地址解析失败或非 HTTP(S) 协议也由业务校验拒绝。

参考 [serde_json::from_str](https://docs.rs/serde_json/1.0.154/serde_json/fn.from_str.html)。

## 解析错误与业务校验错误不同

Serde 负责把数据转换成 Rust 类型，不自动理解所有业务规则：

| JSON 内容 | 反序列化 | 业务上是否一定有效 |
|---|---:|---:|
| 缺少右花括号 | 失败 | 否 |
| `workers` 是字符串 | 失败 | 否 |
| `workers` 是空数组 | 成功 | 不一定 |
| 两个 Worker ID 相同 | 成功 | 不一定 |
| 地址是普通字符串 `hello` | 成功 | 不一定 |

因此后续流程通常是：

```text
JSON 文本
   │ serde_json::from_str
   ▼
结构正确的 GatewayConfig
   │ validate
   ▼
业务上可用的 GatewayConfig
```

解析错误与校验错误可以最终统一为 `ConfigError`，但它们代表不同失败阶段，错误消息也应让
用户能够区分。

## 自查与关联

空 workers 能被解析，为什么仍然不能启动？

[Serialize 与 Deserialize](serialize-and-deserialize.md) · [字段与枚举的 JSON 映射](field-and-enum-mapping.md)
