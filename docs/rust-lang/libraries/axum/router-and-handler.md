# Router 与 Handler：注册和执行是两件事

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**post(chat) 注册处理器，收到请求后框架才提取参数并驱动返回的 Future。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [解构：从 State、结构体和元组中取值](../../concepts/patterns/destructuring.md) · [Future 与 await：创建、暂停和继续](../../concepts/async/future-and-await.md) |
| 源码入口 | [src/main.rs](../../../../src/main.rs)：`.route("/chat", post(chat))` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 按括号层次读代码

用已有测试路由的简化例子观察语法：

```rust
post(move || {
    async move {
        (StatusCode::OK, "successful response")
    }
})
```

```text
post(                               // 函数调用
    move || {                       // 参数：闭包
        async move {                // 闭包返回：Future
            (StatusCode::OK, "...") // Future 完成后的值
        }
    }
)                                   // post 返回：MethodRouter
```

| 语法 | 大白话 |
|---|---|
| `move` | 按值捕获用到的外部变量，不是“开始执行” |
| `\|\|` | 闭包参数列表为空 |
| 外层 `{ ... }` | 闭包体 |
| `async move { ... }` | 创建按值捕获的 Future，是异步块，不是闭包 |
| 最后一个表达式没有分号 | 作为所在块的结果 |

外层最后的表达式是异步块，所以闭包返回 Future。
内层最后的表达式是元组，所以 Future 完成后产出元组。
闭包用 `handler()` 调用；Future 可通过 `.await` 驱动。创建 Future 不等于完成其中的工作。

## 看 post 签名：输入不是返回值

Axum 通过宏生成 post 函数，展开名字后，签名为：

```rust
pub fn post<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
```

这里只摘录签名，不是完整函数实现。

| 部分 | 含义 |
|---|---|
| `handler: H` | 接收一个类型为 H 的处理器 |
| `H: Handler<T, S>` | H 必须满足 Axum 的 Handler 契约 |
| `-> MethodRouter<S, Infallible>` | post 自己的返回类型 |
| `S` | 应用状态的类型 |
| `T` | 区分 Handler 实现的类型参数，本例不展开 |

签名没有要求参数只能是闭包，普通 async 函数也可以作为处理器。
本例判断参数是闭包的直接证据，是使用处写了 `move ||`。

## 看 Handler 约束：为什么调用后返回 Future？

对于本例这种无参数处理器，Handler 实现的关键 where 约束是：

```rust
F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
Fut: Future<Output = Res> + Send,
Res: IntoResponse,
```

这三行属于同一个实现，不是三个独立函数。

```text
F：处理器类型
    │ FnOnce() -> Fut：不带参数调用，得到 Fut
    ▼
Fut：Future 类型
    │ Future<Output = Res>：完成后得到 Res
    ▼
Res：业务返回值类型
    │ IntoResponse：能转换成 HTTP 响应
    ▼
Response
```

| 约束 | 本例对应 |
|---|---|
| `FnOnce() -> Fut` | 外层闭包调用后返回 Future |
| `Future<Output = Res>` | 内层 async move 完成后返回元组 |
| `Res: IntoResponse` | `(StatusCode, &'static str)` 可转换成响应 |

H 和 F 是不同源码位置选用的泛型名字，这里都指处理器类型，不是两个额外对象。
Fut、Res 的实际类型由编译器根据传入值推断。

判断依据有两处：

1. 使用处的 `move ||`：证明传入的是闭包。
2. 库的 Handler 约束：证明该闭包调用后必须返回 Future。

这里摘录的是无参数实现。带 State、Json 等参数时，需要看对应实现及提取器约束。

FnOnce 不表示整个路由只能服务一次：Axum 会克隆处理器，再调用对应副本。
重试测试里，每次调用时 Arc::clone 让本次请求的 Future 持有自己的 Arc，仍共享同一个计数器。
双层捕获见 [闭包与双层 move](../../concepts/async/async-capture.md)。

这里 `Output = Res` 是关联类型等式约束，不是普通泛型参数。
具体区别及重试函数中的 Result 例子见
[类型别名与 Future::Output](../../concepts/types/associated-types.md)。

## 注册路由与处理请求的时间线

```text
构建路由时
  创建闭包 → post(闭包) → MethodRouter → route("/chat", ...) → Router

收到请求时
  匹配 /chat → 匹配 POST → 调用处理器 → 得到 Future
                                            ↓ 运行时驱动
                                       得到业务返回值
                                            ↓ IntoResponse
                                         HTTP 响应
```

Handler 内部的核心步骤是：

```rust
self().await.into_response()
```

| 步骤 | 含义 |
|---|---|
| `self()` | 调用处理器，得到 Future |
| `.await` | 等待 Future 完成，得到返回值 |
| `.into_response()` | 将返回值转换为 HTTP 响应 |

这是框架内部流程，无需在每个处理器里重复写。

## MethodRouter 是什么类型？

它是 Axum 定义的泛型结构体：`MethodRouter<S = (), E = Infallible>`。

| 参数 | 默认类型 | 含义 |
|---|---|---|
| S | `()` | 应用状态，默认无状态 |
| E | `Infallible` | 服务错误类型，默认没有这一层错误 |

内部保存 GET、POST 等方法的处理规则，不是闭包，也不是 Future。
Infallible 不等于永远返回 HTTP 200；HTTP 400、503 仍可以是正常生成的响应。
HTTP 状态码与服务层 Result 的错误类型是两回事。

## 自查与关联

注册路由时 chat 已执行了吗？

[State：框架怎样准备共享状态](state.md) · [用最小模型理解 Extractor](extractor-mechanics.md) · [Json 与 Path：从请求准备参数](json-and-path.md)
