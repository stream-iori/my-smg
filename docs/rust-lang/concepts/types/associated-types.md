# 关联类型：实现确定自己的 Output

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**泛型参数是使用通用定义时填入的类型；关联类型由具体 trait 实现确定，等式约束只检查要求。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) |
| 源码入口 | [src/gateway/retry.rs](../../../../src/gateway/retry.rs)：`Fut: Future<Output = Result<HttpResponse, HttpResponse>>` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 先看“谁确定类型”

```rust
let numbers: Vec<u32> = Vec::new();
let flags: Vec<bool> = Vec::new();
```

创建不同的 Vec 类型时可以填写不同的 T。关联类型则由实现确定：

```rust
let number_future = async { 42_u32 };
let flag_future = async { true };
```

```text
number_future 的具体类型 → 实现 Future → Output = u32
flag_future 的具体类型   → 实现 Future → Output = bool
```

`Future<Output = u32>` 是筛选条件：只接受已经符合要求的实现，不会把第二个 Future 的 bool 改成 u32。泛型与关联类型不是简单的“入参和出参”之分；这里的 Output 恰好用来描述输出。

## Output：Future 的关联类型

Future trait 内声明了 `type Output;`。只看这个声明的概念骨架：

```rust
trait Future {
    type Output;
    // 省略 poll 等定义；这不是完整的标准库源码。
}
```

实现 Future 的具体类型，需要确定自己的 Output。
普通 async 函数和异步块由编译器生成对应的 Future 实现，不需要我们手写。

```rust
async fn answer() -> usize {
    42
}
```

```text
answer()
    ↓ 返回一个编译器生成的 Future 类型
该类型实现 Future，Output = usize
    ↓ .await
42：usize
```

对于直接实现 Future 的类型，`.await` 得到的就是它的 Output。
更一般地，可等待的值也可以先通过 IntoFuture 转成 Future，见 Tokio 专题。

## 怎样显式声明输出类型

**可以显式声明，但要分清：声明 `.await` 后的值类型，还是声明 Future 的输出类型。**
以下是独立语法示例，不是项目实现任务。

**① 在结果变量上标注类型。**

```rust
let number_future = async { 42_u32 };
let number: u32 = number_future.await;
```

`number: u32` 要求等待结果必须符合 u32。它不会把其他类型自动转换成 u32，
不符合就编译报错。Future 的实现仍由编译器根据异步块生成。

**② 用 async 函数声明输出类型，这是常见写法。**

```rust
async fn answer() -> u32 {
    42
}
```

```text
answer()       → 返回编译器生成的 Future
                 该类型实现 Future，Output = u32

answer().await → 得到 u32
```

虽然签名写的是 `-> u32`，调用 `answer()` 并不是立即得到 u32。
**你声明完成后的结果类型，编译器生成对应的 Future 实现。**

**③ 用普通函数明确写出返回 Future 的约束。**

```rust
use std::future::Future;

fn answer() -> impl Future<Output = u32> {
    async { 42_u32 }
}
```

这是②的另一种写法，两个同名函数是替代例子，不应放在同一作用域中重复定义。

| 部分 | 含义 |
|---|---|
| `impl Future` | 返回某个实现了 Future 的具体类型，调用方不必知道其名字 |
| `Output = u32` | 要求这个 Future 实现的输出类型为 u32 |
| `async { ... }` | 编译器生成返回的具体 Future 类型及其实现 |

```text
async fn answer() -> u32
          ≈ 这两个例子的表达意图相同
fn answer() -> impl Future<Output = u32>
```

这里的 `Output = u32` 是编译期约束，不会把 bool 等其他输出强制转换成 u32。

**④ 实现 trait 时，也可以直接给关联类型赋值。**

```text
impl Future for 某个具体类型 {
    type Output = u32;
    ……还必须实现 poll……
}
```

这是不完整的接口形状，只用于展示 `type Output = u32` 出现在哪里。
普通业务代码通常采用②或③，让编译器生成实现，不需要手写 Future 或 poll。

| 写法 | 明确了什么 | 谁生成 Future 实现 |
|---|---|---|
| `let number: u32 = future.await` | 等待后的局部值必须是 u32 | 这条变量声明不生成实现 |
| `async fn answer() -> u32` | 函数生成的 Future 输出 u32 | 编译器 |
| `fn answer() -> impl Future<Output = u32>` | 返回类型必须是输出 u32 的 Future | 本例中的异步块由编译器生成实现 |
| 在具体实现中写 `type Output = u32` | 直接确定该实现的关联类型 | 实现者还需提供 poll |

对应规则见 [Rust Reference：异步函数](https://doc.rust-lang.org/reference/items/functions.html#async-functions)
与 [标准库 Future](https://doc.rust-lang.org/std/future/trait.Future.html)。

## 为什么写 Output =，而不是 Future<usize>？

| 语法 | 给谁指定类型 |
|---|---|
| `Result<usize, String>` | Result 声明的两个泛型参数 |
| `Future<Output = usize>` | Future 声明的关联类型 Output |

Future 本身没有声明一个普通类型参数供 `Future<usize>` 使用。
尖括号不只用于传泛型参数，也可以写关联类型约束，`Output =` 的等号不能省略。

对于给定的具体 Future 实现，Output 已由实现确定。调用者的等式约束是在检查它是否符合要求，
不是让同一个 Future 在调用时随意选择不同的完成结果。

```text
泛型参数：接口上可传入的类型槽位
关联类型：某个 trait 实现确定的类型
```

## 自查与关联

Output = u32 是转换结果还是检查已有实现？

[泛型、类型推断与 turbofish](generics-and-inference.md) · [Trait：类型需要提供什么能力](traits-and-bounds.md) · [类型别名与新类型](aliases-and-newtypes.md)
