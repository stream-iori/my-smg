# String 与 str：拥有文本还是借用文本

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**String 拥有可增长的文本，&str 是借用视图；as_str、to_owned 与 into 的行为不同。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [struct、enum、impl 与 Self](struct-enum-and-impl.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker.rs](../../../../src/worker.rs)：`Worker::id` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 项目入口：节点拥有地址，调用者借用地址

摘自 `src/worker.rs`：

```rust
pub fn address(&self) -> &str {
    &self.address
}
```

Worker 的字段是 String，方法只借出文本视图，调用方不取得字段所有权。

| 表达式 | 类型 | 是否复制文本 |
|---|---|---|
| `worker.address()` | `&str` | 否 |
| `worker.address().to_owned()` | `String` | 是，创建独立拥有值 |
| `String::from("Alice")` | `String` | 是 |
| `"Alice".to_owned()` | `String` | 是 |

```text
worker.address：String ──拥有──> 地址文本
                                  ↑
address() 返回 &str ──借用─────────┘

.to_owned() → 复制内容 → 新的 String
```

`to_owned` 可以读作“得到拥有型的值”。这里与 `String::from` 效果相同；其他类型的 to_owned 要看相应实现。

## String as str vs borrow

**`url.as_str()` 与 `&url` 的区别**

前提：这里的 `url` 是 `String`，不是 `reqwest::Url`。

| 表达式 | 类型 | 借用什么 | 复制字符串内容吗 |
|---|---|---|---|
| `url.as_str()` | `&str` | 字符串内容的视图 | 不复制 |
| `&url` | `&String` | String 对象 | 不复制 |
| `url.clone()` | `String` | 不借用，创建独立的拥有者 | 复制 |

```rust
let url = String::from("http://127.0.0.1:3001/chat");
let text: &str = url.as_str();
let object: &String = &url;
```

```text
url: String ──拥有──> 字符串内容
      ▲                  ▲
      │                  │
 &url: &String      url.as_str(): &str
```

两者都不转移 url 的所有权；借用使用期间，原值必须仍然有效。

## UTF-8：字节长度不等于字数

String 与 str 保存 UTF-8 文本，len 统计字节。“你好”是 6 字节、两个 Unicode 标量值。
Rust char 表示 Unicode 标量值，不一定是用户看到的一个完整字符。

| 需求 | 边界 |
|---|---|
| 用整数下标取字符 | String 不支持这种操作 |
| 用字节范围切片 | 必须位于 UTF-8 字符边界，否则下标切片会 panic |
| 遍历标量值 | 使用 chars，不把它当成通用可见字符分割器 |

参见[Rust 官方字符串章节](https://doc.rust-lang.org/book/ch08-02-strings.html)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | address() 与 address().to_owned() 哪个获得独立拥有值？ |
| 解释 | &String 与 &str 为什么不同，但都不接管原文本？ |
| 边界 | 原 String 销毁后，借用其内容的 &str 还能继续使用吗？ |

先写下判断和原因，再用自己的已有代码验证。

[struct、enum、impl 与 Self](struct-enum-and-impl.md) · [元组、元组结构体与 unit](tuples-and-unit.md) · [表达式、返回值与分号](expressions-and-semicolons.md)
