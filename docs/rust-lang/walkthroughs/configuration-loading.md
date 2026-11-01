# 代码导读：配置加载与分层错误

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**文件读取、结构解析、业务校验、运行时组装各有责任和错误边界。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [? 与 ??：成功继续，失败提前返回](../concepts/errors/question-mark.md) · [错误枚举、Error 与 source](../concepts/errors/error-and-source.md) · [JSON 解析与业务校验](../libraries/serde/json-parsing-and-validation.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/config.rs](../../../src/config.rs)：`GatewayConfig::from_file` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 实际调用链与责任

```text
main
  ↓ GatewayConfig::from_file
读取文件 → from_json_validated
               ├── from_json：结构解析
               └── validate：业务规则
  ↓
构造 Registry / Policy / GatewayState
  ↓
注册路由并启动服务
```

| 失败阶段 | 当前错误 |
|---|---|
| 文件读取 | ConfigLoadError::Read |
| JSON 解析 | ConfigLoadError::Parse |
| 校验 | ConfigLoadError::Validation |

main 决定启动错误的文字和退出码；底层保存具体错误，不直接 process::exit。
当前 main 还有重复 ID 的防御检查：即使配置已经校验过，插入失败仍会返回启动错误。

## 从 JSON 配置解析开始

下面的关联函数尝试把 JSON 文本解析成 `GatewayConfig`：

```rust
impl GatewayConfig {
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }
}
```

当前 `impl GatewayConfig` 中，`Self` 就是 `GatewayConfig`，所以返回类型等价于：

```rust
Result<GatewayConfig, serde_json::Error>
```

它有两种可能：

```text
Ok(GatewayConfig)      JSON 有效，成功得到配置
Err(serde_json::Error) JSON 语法或字段类型不符合要求
```

这里不需要 `?`，因为 `serde_json::from_str(input)` 返回的类型正好就是当前函数要返回的类型，
直接返回最简洁。

## “交给调用者”到底是什么意思

假设调用关系是：

```text
main
  └─ start_gateway
       └─ load_config
            └─ GatewayConfig::from_json
```

如果最底层解析失败：

```text
from_json 返回 Err
       │
       │ load_config 使用 ?
       ▼
load_config 提前返回 Err
       │
       │ start_gateway 也使用 ?
       ▼
start_gateway 提前返回 Err
       │
       ▼
main 最终决定如何打印错误以及用什么退出码结束
```

每一层都可以选择：

1. 当场处理并恢复；
2. 把底层错误转换为更符合本层语义的错误；
3. 使用 `?` 继续交给上一层。

因此，`?` 表达的是“这层没有足够上下文恢复，请上层决定”，而不是“错误不重要”。

## 多个步骤如何连续传播错误

一个真实的配置加载流程可能包含：

```text
读取文件 -> 解析 JSON -> 校验业务规则 -> 构造运行时对象
```

伪代码可以写成：

```rust
fn load(path: &Path) -> Result<GatewayConfig, ConfigError> {
    let text = read_file(path)?;
    let config = parse_json(&text)?;
    validate(&config)?;
    Ok(config)
}
```

任意一步失败都会提前返回，后面的步骤不会执行。例如读取失败时，不会继续解析；解析失败
时，也不会拿一个不存在的配置去校验。

这比多层嵌套 `match` 更容易看出“成功主线”。

## 统一例子：配置加载流水线

```text
JSON 文本
   │ 解析
   ├────────失败────> serde_json::Error
   ▼
GatewayConfig
   │ 校验
   ├────────失败────> ConfigError
   ▼
可运行配置

高层统一入口：
ConfigLoadError
├─ Parse(serde_json::Error)
└─ Validation(ConfigError)
```

| 类型 | 负责什么 |
|---|---|
| `serde_json::Error` | JSON 语法和字段类型错误 |
| `ConfigError` | 空节点、重复 ID、非法地址等业务错误 |
| `ConfigLoadError` | 为完整配置加载提供统一错误边界 |

## 阶段不同，错误语义不同

同一句“配置错了”可能对应不同事实：

| 输入 | 解析 | 校验 | 错误阶段 |
|---|---:|---:|---|
| 缺少 `}` | 失败 | 不执行 | Parse |
| `workers` 是字符串 | 失败 | 不执行 | Parse |
| `workers` 是空数组 | 成功 | 失败 | Validation |
| Worker ID 重复 | 成功 | 失败 | Validation |

原则：

```text
先问“在哪一步失败”，再设计错误类型。
```

如果所有错误都叫 `InvalidConfig`，调用者很难给出准确提示，也难以决定是否重试或修复输入。

## 自查与关联

哪一层决定错误怎样打印，哪一层保留底层原因？

[代码导读：Worker 与在途请求的生命周期](worker-lifecycle.md) · [代码导读：轮询选择与策略状态](policy-selection.md) · [代码导读：一次聊天请求](chat-forwarding.md)
