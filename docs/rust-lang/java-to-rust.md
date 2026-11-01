# 从 Java 经验转到 Rust

[总入口](index.md) · [阶段阅读清单](stage-reading.md) · [问题索引](questions.md)

**先判断值归谁、谁能访问、何时释放，再对照语法。** 本页是迁移地图，不增加课程阶段；只读当前任务对应行。

| 阶段 | Java 经验 | Rust 需要建立的区别 | 详细阅读 |
|---|---|---|---|
| 1～2 | 赋值或传参后，多个变量可以继续引用同一对象 | 按类型区分 Copy、move、借用；赋值不一定增加一个仍可用的别名 | [Move](concepts/ownership/move-and-partial-move.md)、[借用](concepts/ownership/borrowing.md)、[Copy/Clone](concepts/ownership/copy-and-clone.md)基础部分 |
| 1～2 | final 限制重新赋值，不保证对象内部不可变 | let mut 控制绑定可变性，&mut 表达一次独占访问；共享引用也不等于整个对象永远不可变 | [方法接收者](concepts/ownership/method-receivers.md)；内部可变性阶段 7 再学 |
| 1～3 | GC 回收内存，外部资源另用 close 或 try-with-resources | 拥有型值通常在离开作用域时释放资源，move 转移释放责任，借用不增加拥有者 | [所有权](concepts/ownership/move-and-partial-move.md)、[Drop/RAII](concepts/ownership/drop-and-raii.md)基础部分 |
| 1～2 | Java String 不可变，长度与索引按 UTF-16 代码单元处理 | String 拥有可增长的 UTF-8 文本，&str 借用文本；len 是字节数，不支持整数下标直接取字符 | [String/str](concepts/values/string-and-str.md) |
| 2、4 | 无值可能用 null，失败可能抛异常 | Option 显式表达有无，Result 显式表达成败；panic 不是普通的错误返回值 | [Option/Result](concepts/errors/option-and-result.md)、[unwrap/expect](concepts/errors/unwrap-and-expect.md) |
| 5、9 | CompletableFuture 常由已提交的计算或其他操作完成 | 调用 async fn 先得到 Future，轮询才执行函数体；spawn 提交独立任务，await 等待结果 | [Future/await](concepts/async/future-and-await.md)、[spawn](libraries/tokio/spawn.md) |

## 用一张图检查数据交接

```text
拿到一个 Rust 值
    ├─ 只临时使用 → 借用 → 原拥有者继续负责资源
    ├─ 接管后续处理 → 按值交接 → 非 Copy 值转移所有权
    └─ 需要另一个值 → 检查 Copy / Clone 的具体语义与成本
```

Java 的引用值也是按值传递，不能把 Java 概括成“对象按引用传递”。Rust 的引用同样是值，但带有访问权限和有效范围约束。

## 三个容易套错的结论

| 套用的结论 | 应检查什么 |
|---|---|
| clone 就是深拷贝 | 具体类型的实现；阶段 7 的 Arc::clone 只增加共享拥有句柄 |
| 标注更长生命周期就能让对象多活一会儿 | 谁拥有数据；标注只表达借用关系，不延长资源存活 |
| async 自动创建线程 | Future 是否被驱动、是否提交任务；异步函数中仍可能执行阻塞操作 |

Rust 的普通所有权管理不依赖 Java 式追踪 GC；引用计数也不等于 GC。正常离开作用域和强制结束进程要分开判断。
参见[Rust 官方所有权说明](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)、[UTF-8 字符串](https://doc.rust-lang.org/book/ch08-02-strings.html)、[Future 协议](https://doc.rust-lang.org/std/future/trait.Future.html)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测（阶段 1～2） | String 按值交给另一个变量后，原变量还能像 Java 对象别名一样使用吗？ |
| 解释（阶段 1～2） | 为什么修改权限与释放责任要分开判断？ |
| 边界（阶段 1～2） | 不可变绑定与整个对象图不可修改，是同一回事吗？ |

后续阶段再回答：Java 异步经验能直接决定 Rust Future 的启动和取消时机吗？

返回[阶段清单](stage-reading.md)。Maven 多模块、JPMS、Netty、PECS 均为可选对照，不作为 T00～T07 前置。
