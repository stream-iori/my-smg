# match 分支与守卫的执行顺序

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**从上往下匹配，只执行第一个模式和守卫都成功的分支。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Option 与 Result：有无与成败](../errors/option-and-result.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway/tests/support.rs](../../../../src/gateway/tests/support.rs)：`TestServer::stop` |
| 阅读时机 | T04 读到基础自查；任务清理在阶段 8～9 阅读。 |

## 从上到下选择一个分支

模式检查变体和内容，守卫添加布尔条件。Some(number) 匹配 Some 并绑定内部整数。
模式与守卫都成功才进入分支；match 必须覆盖所有可能情况。

先用整数看分支顺序：

```rust
let value = Some(-2);
let label = match value {
    Some(number) if number > 0 => "positive",
    Some(_) => "non-positive",
    None => "missing",
};
assert_eq!(label, "non-positive");
```

```text
Some(-2)
   ↓ 第一个模式匹配成功：number = -2
number > 0 为 false
   ↓ 继续尝试第二个分支
Some(_) 匹配成功 → 返回 non-positive
```

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | Some(-2) 为什么得到 non-positive？ |
| 解释 | 模式已匹配但守卫为 false，为什么继续尝试下一分支？ |
| 边界 | 如果只保留带守卫的 Some 分支与 None 分支，负数如何被覆盖？ |

先写下判断和原因，再用自己的已有代码验证。

## 项目应用：阶段 8～9 的任务清理

另需[Result](../errors/option-and-result.md)、[Future](../async/future-and-await.md)、[取消](../../libraries/tokio/cancellation.md)。

## match 分支守卫：模式后面还可以加条件

`my-smg` 的测试服务器清理函数有这一段：

```rust
match self.task.cancel_and_join().await {
    Ok(()) => Ok(()),
    Err(error) if error.is_cancelled() => Ok(()),
    Err(error) => Err(error),
}
```

`Err(error) if error.is_cancelled()` 分两步：先匹配 Err 并绑定 error，再检查布尔条件。
条件不满足时，继续尝试后面的分支；不是直接退出 match。

| 输入 | 选中的分支 | 清理结果 |
|---|---|---|
| 正常完成 | `Ok(())` | 成功 |
| 因取消结束 | 带 if 的 Err 分支 | 成功 |
| panic 等其他任务错误 | 最后一个 Err 分支 | 保留错误 |

普通模式负责匹配结构，分支守卫负责附加条件。守卫可以为 false，因此不能只靠一个带守卫的
分支覆盖所有 Err；这里保留无条件的最后一个分支。
参见 [Rust Reference：match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards)。

这段逻辑适用于“服务器清理允许已完成”的场景。取消行为测试仍需要明确检查
`JoinError::is_cancelled()`，不能用“清理成功”替代“确实被取消”的断言。见
[Tokio 资源管理](../../walkthroughs/task-cleanup.md)。
