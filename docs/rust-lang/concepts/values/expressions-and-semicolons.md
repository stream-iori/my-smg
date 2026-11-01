# 表达式、返回值与分号

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**表达式产生值；块末尾的分号通常会丢弃该表达式的值。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | 无；认识整数、布尔值和函数输入输出即可。 |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：策略初始化；[src/gateway.rs](../../../../src/gateway.rs)：selected_idex |
| 阅读时机 | T04 读到“基础自查”；项目应用在阶段 4 后阅读。 |

## 块会得到什么

| 写法 | 结果 |
|---|---|
| `1 + 2` | 产生整数 3 |
| `{ 1 + 2 }` | 块的末尾表达式提供整数 3 |
| `{ 1 + 2; }` | 结果被丢弃，块得到 unit 值 `()` |
| `return` 后带一个值 | 立即向调用方返回该值，结束当前函数 |

函数签名的 `->` 指定返回类型。正常执行到函数体末尾时，由末尾表达式提供返回值。
声明返回整数却得到 unit 时，编译器会报告类型不匹配。

## match 也是表达式

match 从分支中选择结果，沿用已有独立语法例子：

```rust
fn choose(flag: bool) -> i32 {
    match flag {
        true => 1,
        false => 2,
    } // 没有分号：整个 match 的值作为函数返回值
}
```

函数中的末尾 match 提供整数。如果在它后面添加分号，结果会被丢弃。
`let x = ...;` 的分号则结束声明，右边的结果已经存入 x。unit 见[元组与 unit](tuples-and-unit.md)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | choose 收到 false 时返回什么？ |
| 解释 | 为什么末尾表达式与 let 声明末尾的分号效果不同？ |
| 边界 | 给上述 match 后添加分号，还满足返回 i32 的约定吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

## 项目应用：阶段 4 后再读

先读[借用](../ownership/borrowing.md)、[trait](../types/traits-and-bounds.md)和[动态分发](../types/static-and-dynamic-dispatch.md)；Send 在阶段 7 回查[Send/Sync](../sharing/send-and-sync.md)。
以下摘自当前 main.rs：

```rust
let policy: Box<dyn Policy + Send> = match &config.policy {
    PolicyKind::FirstHealthy => Box::new(FirstHealthy::new()),
    PolicyKind::RoundRobin => Box::new(RoundRobin::new()),
};
```

`&config.policy` 等价于 `&(config.policy)`；分支的结果用于初始化 policy，末尾分号结束 let 声明。

**PolicyKind 没有实现 Copy，不代表 `match config.policy` 一定移动该字段。**
当前两个变体均不带字段；只匹配它们、不按值绑定或移出内容时，不会取走 policy。
按值绑定非 Copy 内容时才可能发生移动，不能只看有没有 `&`。
见[Rust Reference：匹配对象与绑定](https://doc.rust-lang.org/reference/expressions/match-expr.html#scrutinee-behavior)。

当前 GatewayState 保存 client、registry、policy，不保存完整 config。
chat 当前用 let-else 处理选中索引，不应套用旧版选择节点的 match 叙述。

综合题：只判断变体，与按值取出变体携带的 String，所有权效果为什么不同？
