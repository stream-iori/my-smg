# 用最小模型理解 Extractor

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**类型约束选择提取规则，运行时执行规则得到参数值；提取不同于模式解构。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](../../concepts/types/generics-and-inference.md) · [Trait：类型需要提供什么能力](../../concepts/types/traits-and-bounds.md) · [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`chat` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 最小例子

```rust
// 1. 你定义的应用数据
#[derive(Clone)]
struct AppState {
    name: String,
}

// 2. 框架定义的包装类型
struct State<T>(T);

// 3. 框架约定：参数必须说明怎样从应用状态中取得自己
trait FromState<S> {
    fn from_state(source: &S) -> Self;
}

// 4. State<T> 的提取规则：克隆数据，再包装
impl<T: Clone> FromState<T> for State<T> {
    fn from_state(source: &T) -> Self {
        State(source.clone())
    }
}

// 5. 模拟框架：准备参数，然后调用你的函数
fn dispatch<S, E: FromState<S>>(
    source: &S,
    handler: fn(E),
) {
    let argument = E::from_state(source);
    handler(argument);
}

// 6. 你只声明自己需要 State<AppState>
fn health(wrapper: State<AppState>) {
    println!("{}", wrapper.0.name);
}

fn main() {
    let state = AppState {
        name: String::from("gateway"),
    };

    dispatch(&state, health);
}
```

独立编译运行此例，输出为 `gateway`；这不涉及启动 HTTP 服务。

## trait 怎样描述提取规则

```rust
trait FromState<S> {
    fn from_state(source: &S) -> Self;
}
```

| 部分 | 大白话 |
|---|---|
| `S` | 数据来源的类型 |
| `source: &S` | 借用来源数据，不直接把它搬走 |
| `Self` | 实现这个 trait 的目标类型 |
| `-> Self` | 返回一个准备好的目标值 |

`from_state` 没有 `self` 参数，它是关联函数，不需要先创建一个目标对象才能调用。

接着为包装类型实现它：

```rust
impl<T: Clone> FromState<T> for State<T> {
    fn from_state(source: &T) -> Self {
        State(source.clone())
    }
}
```

对本例而言，T 是 AppState：

```text
来源类型：T = AppState
目标类型：Self = State<AppState>

&AppState → 克隆出 AppState → 包装成 State<AppState>
```

因为只拿到了来源的共享引用，不能把里面的 AppState 直接移动出去，所以本例使用 Clone 创建一个值。
这里克隆 String 会复制字符串数据；若状态字段是 Arc，克隆句柄则可以继续共享底层对象。
“提取”不等于“永远克隆”，这是本例选定的具体规则。

## dispatch 如何知道该调用哪个实现

最关键的一次调用是：

```rust
dispatch(&state, health);
```

把它和函数签名对上：

```rust
fn dispatch<S, E: FromState<S>>(
    source: &S,
    handler: fn(E),
)
```

| 信息来源 | 编译器得到什么 |
|---|---|
| `state` 的类型是 AppState | `S = AppState` |
| `health` 接收 State<AppState> | `E = State<AppState>` |
| `E: FromState<S>` | 必须存在 `State<AppState>: FromState<AppState>` 的实现 |

health 是函数项，传给这里的 `fn(E)` 参数时会转换成对应的函数指针。本例不支持捕获环境的闭包。

类型确定后，下面这一行：

```rust
let argument = E::from_state(source);
```

在本次调用中对应：

```rust
let argument = State::<AppState>::from_state(source);
```

如果要明确指出 trait，可以写成完整限定形式：

```rust
let argument =
    <State<AppState> as FromState<AppState>>::from_state(source);
```

这三段是在解释同一次调用，不是需要重复执行三次的步骤。

```text
dispatch(&state, health)
    │
    ├── 根据 &state 确定 S
    ├── 根据 health 的参数确定 E
    ├── 检查 E 是否实现 FromState<S>
    │
    v
E::from_state(source)
    │
    v
State(source.clone())
    │
    v
health(argument)
```

**提取规则在编译期确定，具体数据在运行时取得。**
程序不需要扫描参数名、解析类型字符串或使用 Java 式运行时反射。
如果去掉这个 trait 实现，原来的 dispatch 调用无法通过编译；它不会等到请求到来后才猜该怎么办。

## 提取和解构是两步

| 步骤 | 对应代码 | 做什么 |
|---|---|---|
| 提取 | `E::from_state(source)` | 根据来源生成包装参数 |
| 调用 | `handler(argument)` | 把参数传给业务函数 |
| 访问字段 | `wrapper.0` | 从收到的包装里取出内部值 |

health 可以改写成先解构的形式：

```rust
fn health(wrapper: State<AppState>) {
    let State(state) = wrapper;
    println!("{}", state.name);
}
```

也可以在参数位置直接解构：

```rust
fn health(State(state): State<AppState>) {
    println!("{}", state.name);
}
```

这里最后一种写法更短，但初学时可以保留 `wrapper` 和 `.0`，分开阅读提取与取值。
具体语法见 [State(state)：解构元组结构体](../../concepts/patterns/destructuring.md)。

## 自查与关联

只定义 State 结构体是否就具备提取能力？

[Router 与 Handler：注册和执行是两件事](router-and-handler.md) · [State：框架怎样准备共享状态](state.md) · [Json 与 Path：从请求准备参数](json-and-path.md)
