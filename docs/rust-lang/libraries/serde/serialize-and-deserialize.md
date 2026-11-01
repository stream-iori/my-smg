# Serialize 与 Deserialize

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**序列化把值交给格式输出，反序列化从格式数据构造类型；trait 不等同于 JSON 格式本身。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [struct、enum、impl 与 Self](../../concepts/values/struct-enum-and-impl.md) · [derive 与 Default：编译器替你生成什么](../../concepts/types/derive-and-default.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`#[derive(Deserialize, Serialize)]` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## Serde：`Serialize` 与 `Deserialize`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WorkerConfig {
    model_id: String,

    #[serde(default)]
    enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
}
```

```text
Serialize：Rust 值 -> JSON / YAML / MessagePack 等数据模型
Deserialize：外部数据 -> Rust 值
```

常用辅助属性：

| 属性 | 作用 |
|---|---|
| `#[serde(rename = "x")]` | 修改单个字段或 variant 的外部名称 |
| `#[serde(rename_all = "snake_case")]` | 批量修改命名规则 |
| `#[serde(default)]` | 字段缺失时使用 `Default` |
| `#[serde(skip)]` | 序列化和反序列化都跳过 |
| `#[serde(skip_serializing_if = "...")]` | 满足条件时不输出字段 |
| `#[serde(flatten)]` | 将嵌套结构展开到当前对象 |

派生 Serde trait 相当于定义外部数据协议。重命名字段、修改 enum 表示形式或增加必填字段，都可能
是兼容性变更。

SGLang 中的配置类型大量使用这一组合，见
[`config/types.rs`](../../../../../sgl-model-gateway/src/config/types.rs)。

## 自查与关联

结构体字段公开与能否序列化是否是同一回事？

[字段与枚举的 JSON 映射](field-and-enum-mapping.md) · [JSON 解析与业务校验](json-parsing-and-validation.md)
