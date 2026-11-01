# TryFrom、TryInto 与可能失败的转换

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**需要检查范围或业务条件的转换返回 Result，失败信息留给调用方。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [From 与 Into：为什么有两个转换入口](from-and-into.md) · [Option 与 Result：有无与成败](option-and-result.md) · [类型推断、parse、collect 与空 Vec](../types/type-inference.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`parse::<u64>` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 可能失败的转换

| trait / 方法 | 返回类型 | 适用情况 |
|---|---|---|
| `From` / `from()` | 目标值 | 转换不通过 Result 报告失败 |
| `Into` / `into()` | 目标值 | 同上，从源值调用 |
| `TryFrom` / `try_from()` | `Result<目标值, 错误>` | 转换可能失败 |
| `TryInto` / `try_into()` | `Result<目标值, 错误>` | 同上，从源值调用 |

```rust
let value: Result<u8, _> = 300_u16.try_into();
assert!(value.is_err()); // 300 超出了 u8 的范围。
```

实现 TryFrom 会获得对应的 TryInto。From / Into 应用于语义合理、不丢失信息且不主动失败的
转换；需要检查范围或业务约束时，选择 TryFrom / TryInto。

## 自查与关联

parse 和 try_into 分别转换什么来源？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [From 与 Into：为什么有两个转换入口](from-and-into.md)
