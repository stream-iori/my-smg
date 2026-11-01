# 协变与不变：从 Java 泛型到 Rust 引用

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**变型传播子类型关系，不表示任意类型转换；可变引用对内层类型不变。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Java 泛型与 PECS：变型的入门对照](java-variance.md) · [生命周期：引用不能比数据活得久](../concepts/ownership/lifetimes.md) |
| 源码入口 | [src/worker.rs](../../../src/worker.rs)：`InFlightGuard<'a>` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 先抓住一个问题

**里面的类型可以替换，外面的容器能不能跟着替换？这就是变型（variance）研究的问题。**

本文用 `A → B` 表示：需要 `B` 的地方，可以提供 `A`。箭头表示子类型替换，不代表执行任意类型转换。

| 关系 | 已知 `A → B`，套上 `F` 后怎样 | 大白话 |
|---|---|---|
| 协变 | `F<A> → F<B>` | 同方向替换 |
| 逆变 | `F<B> → F<A>` | 反方向替换 |
| 不变 | 不能由此推出上述任一方向 | 不能跟着替换参数 |

推荐按顺序读：

```text
Java：猫和动物 → extends / super → Rust：长引用和短引用
                                            │
                                            v
                             Vec<T> → &T / &mut T → 两层引用
```

下面的代码都是独立语法示例，不是 my-smg 的实现任务。标注“拒绝”的片段用于展示类型检查边界。

## Rust：生命周期提供了另一种替换关系

Java 常用子类和父类演示子类型；Rust 没有类继承，最常见的例子是引用的有效范围。

假设 `'long` 至少和 `'short` 一样长，Rust 写作 `'long: 'short`：

```text
&'long str → &'short str

一个长期有效的引用，可以交给只需要短期使用的地方。
一个短期有效的引用，不能承诺长期有效。
```

生命周期是编译期的有效范围约束，不是秒数，不会延长数据的存活时间。

```rust
fn print(text: &str) {
    println!("{text}");
}

fn main() {
    let text: &'static str = "hello";
    print(text); // 长期有效的引用，可以只在本次调用中使用
}
```

## Rust 泛型也有变型

**“对泛型参数 T 协变”和“通过生命周期举例”可以同时成立：T 本身可以是引用类型。**

例如：

```rust
fn shorten_vec<'a>(values: Vec<&'static str>) -> Vec<&'a str> {
    values
}
```

```text
参数 T：     &'static str →     &'a str
                    │              │
                    v              v
整个类型：Vec<&'static str> → Vec<&'a str>
```

没有逐个重新创建字符串；这里发生的是满足约束的类型替换。`Vec<T>` 对 T 协变。

## 常见类型对 T 的关系

| 类型 | 对 T 的关系 | 关键点 |
|---|---|---|
| `Option<T>` | 协变 | 元素的子类型关系可以传递到外层 |
| `Vec<T>` | 协变 | 同上 |
| `Box<T>` | 协变 | 同上 |
| `&T` | 协变 | 不能通过这个共享引用直接替换 T |
| `&mut T` | 不变 | 可以往原位置写入新的 T |
| `Cell<T>` | 不变 | 可以通过共享引用替换其中的值 |
| `fn(T)` | 逆变 | T 出现在函数输入位置 |
| `fn() -> T` | 协变 | T 出现在函数输出位置 |

不能简单记成“可修改的容器都不变”：`Vec<T>` 自身协变，但通过 `&mut Vec<T>` 修改时，
外层 `&mut` 会要求里面的整个 `Vec<T>` 保持不变。

用户定义结构体的变型由字段使用参数的位置推导；同一参数同时出现在协变和逆变位置时，会成为不变。

## Rust trait 约束不是 Java 的类继承

| Java | Rust | 能否直接照搬 |
|---|---|---|
| `Cat extends Animal` | 无对应的类继承 | 不能 |
| `T extends Animal` 泛型约束 | `T: Animal` trait 约束 | 能类比“要求能力”，不能类比所有子类型规则 |
| `? extends T` / `? super T` | 无直接对应通配符 | 不能逐字翻译 |
| 使用通配符表达边界 | 按字段推导类型的变型 | 机制不同 |

例如以下 trait 只说明 Dog 实现了某种能力：

```rust
trait Animal {}
struct Dog;
impl Animal for Dog {}
```

`Box<Dog>` 可以强制转换成 `Box<dyn Animal>`，但这不是 Java 式子类关系，也不能自动套用到 Vec：

```rust
let dogs: Vec<Box<Dog>> = vec![Box::new(Dog)];
let animals: Vec<Box<dyn Animal>> = dogs; // 拒绝
```

需要逐个转换：

```rust
let animals: Vec<Box<dyn Animal>> = dogs
    .into_iter()
    .map(|dog| dog as Box<dyn Animal>)
    .collect();
```

这不否定 `Vec<T>` 协变：协变传播的是已有的子类型关系，而不是任意强制转换关系。
Rust 的子类型关系还包含更高阶生命周期相关情况，不能说“只有生命周期长短”这一种。

## 自查与关联

外层借用可以缩短，为何不能随意替换内层引用类型？

[高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md) · [扩展：thiserror 的错误派生](thiserror.md)
