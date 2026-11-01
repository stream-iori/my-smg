# 解构：从 State、结构体和元组中取值

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**模式拆开已有值并绑定名字；不会替你获取请求参数。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [struct、enum、impl 与 Self](../values/struct-enum-and-impl.md) · [元组、元组结构体与 unit](../values/tuples-and-unit.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：State(state) |
| 阅读时机 | 基础解构按需阅读；Axum 应用在阶段 5～6。 |

## 先拆普通值

| 已有值 | 模式形状 | 绑定依据 |
|---|---|---|
| 二元组 | `let (left, right) = pair;` | 元素位置 |
| 普通结构体 | `let Worker { id, .. } = worker;` | 字段名 |
| 元组结构体 | `let Wrapper(value) = wrapper;` | 类型构造器名与字段位置 |

这些是形状示意，假定类型存在且字段可见，不是项目完整实现。

```text
pair 的第 0 项 → left
pair 的第 1 项 → right
```

左侧模式负责拆与绑定，右侧表达式提供值。绑定非 Copy 字段可能移动它；
借用的写法见[引用模式](reference-patterns.md)。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 二元组的第 1 项交给哪个名字？ |
| 解释 | 字段名和位置分别用于哪种解构？ |
| 边界 | 三元组能直接被两个位置的模式完整匹配吗？ |

先写下判断与原因，再用自己已有的代码验证；遇到疑问回查本篇，不需要提前读后续应用。

## 项目应用：阶段 5～6 后再读

另外需要[泛型](../types/generics-and-inference.md)、[Future](../async/future-and-await.md)和[路由职责](../../libraries/axum/router-and-handler.md)。
下文 AppState 和 health 是独立形状示意，不代表当前项目的同名 handler。

## `State(state)`：解构元组结构体

Axum 的 `State` 可以简化理解为一个只有一个字段的元组结构体：

```rust
pub struct State<S>(pub S);
```

同样的外形在不同位置有不同含义：

```rust
State(app_state) // 表达式：创建一个 State 值
State(state)     // 模式：拆开 State，把内部值绑定到 state
```

在 Axum handler 中：

```rust
async fn health(State(state): State<AppState>) {
    // state: AppState
}
```

冒号左右分别是：

```text
State(state) : State<AppState>
     │                │
     │                └─ 参数的类型
     └─ 参数使用的解构模式
```

它相当于先接收包装值，再手动拆开：

```rust
async fn health(wrapper: State<AppState>) {
    let State(state) = wrapper;
}
```

完整数据流是：

```text
Router 中的 AppState
        │ Axum State extractor
        v
State<AppState>
        │ State(state) 解构
        v
state: AppState
```

这里的“extractor”和“解构”不是一回事：

| 层次 | 负责什么 |
|---|---|
| Axum `State` extractor | 从 Router 的应用状态构造 `State<AppState>` 参数 |
| Rust `State(state)` 模式 | 从这个元组结构体中取出内部值 |

框架怎样根据参数类型选择提取规则，见 [Extractor 原理：State、泛型与 trait 的最小例子](../../libraries/axum/extractor-mechanics.md)。

默认绑定方式是按值绑定。因为 handler 收到的是一个拥有所有权的 `State<AppState>`，所以
`State(state)` 会把内部 `AppState` 移动给局部变量 `state`，而不是借用它。

## State 是什么类型

```rust
struct State<T>(T);
```

它是一个泛型元组结构体。字段没有名字，通过 `.0` 访问。

| 写法 | 含义 |
|---|---|
| `State<T>` | 内部字段类型为 T 的包装类型 |
| `State<AppState>` | 内部字段类型为 AppState |
| `State(data)` | 表达式：创建包装值，把 data 放进去 |
| `wrapper.0` | 访问包装值的第一个字段 |
| `let State(data) = wrapper;` | 模式解构：拆开包装，把内部值绑定给 data |

```text
wrapper: State<AppState>
└── .0: AppState
    └── name: String
```

只定义这个结构体，还不会自动获得“提取”能力。规则来自后面的 trait 实现。


综合题：框架构造参数和 Rust 模式拆开参数，分别发生在哪一步？
