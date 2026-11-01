# unwrap、expect 与可恢复失败

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**unwrap 和 expect 在失败时 panic，不能代替业务错误处理；expect 用于说明应成立的不变量。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`expect("managed task should own its handle")` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `?` 与 `unwrap`、`expect` 的区别

| 写法 | 失败时行为 | 适合场景 |
|---|---|---|
| `?` | 提前返回 Err 或 None | 传播错误或缺失结果 |
| `match` | 当前层明确处理每个分支 | 需要恢复、分类或自定义行为 |
| `unwrap()` | panic | 已有严格不变量，失败代表程序错误 |
| `expect("...")` | 带消息 panic | 测试，或失败确实不可能且需说明原因 |

测试有效输入时可以使用：

```rust
let config = GatewayConfig::from_json(input)
    .expect("valid config should parse");
```

因为测试目标就是“这份输入必须成功”，失败时 panic 能让测试立刻失败。但正式启动路径不能因
用户配置错误而随意 `unwrap()`，应返回清晰错误。

## 自查与关联

正式配置加载为什么用 Result 而不是 expect？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
