# 两层引用：外层生命周期与内层类型

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**&mut 对外层借用时间协变，对可写入的内层类型不变。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [协变与不变：从 Java 泛型到 Rust 引用](variance.md) |
| 源码入口 | [src/worker.rs](../../../src/worker.rs)：`InFlightGuard<'a>` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## 为什么 &mut T 对 T 不变，却对外层生命周期协变

先固定 T 为 i32，只比较外层借用时间：

```text
&'long i32     → &'short i32       可以
&'long mut i32 → &'short mut i32   可以
```

下面两个函数都能编译：

```rust
fn shorten<'long: 'short, 'short>(x: &'long i32) -> &'short i32 {
    x
}

fn shorten_mut<'long: 'short, 'short>(x: &'long mut i32) -> &'short mut i32 {
    x
}
```

两边指向的数据类型始终是 i32，变化的是“交出去借用多久”。日常代码中的可变引用短期使用也常通过重新借用完成。

要演示里面的 T 能否变化，就需要让 T 自身包含生命周期。两层引用因此出现。

## 先读懂 &&str 和 *slot

```rust
let text: &str = "hello";
let slot: &&str = &text;
```

```text
slot ──引用──> text ──引用──> "hello"
 &&str          &str            str
                 ↑
               *slot
```

| 表达式 | 类型 | 含义 |
|---|---|---|
| `text` | `&str` | 字符串引用 |
| `&text` / `slot` | `&&str` | 对这个字符串引用再借用一次 |
| `*slot` | `&str` | 解开外层，取得里面的引用值 |
| `**slot` | `str` | 再解一层，得到字符串切片 place；不能直接作为普通局部值搬走 |

`*slot` 取得的是可复制的共享引用，不会复制或释放字符串内容。
`println!("{}", slot)` 和 `println!("{}", *slot)` 在这里都能显示 hello，格式化支持引用层层转发。

## 把两个生命周期分别标出来

```text
&'outer mut &'inner str
└─────────┘ └────────┘
 外层借用       T

'outer：借用“保存字符串引用的变量”多久
'inner：里面的字符串引用有效多久
```

`&&'a str` 可以展开理解为 `&'outer &'a str`：外层也有生命周期，只是在原写法里省略了。
合法的两层借用还要求里面的数据在外层使用期间有效。

## 共享引用：里面的长引用可以按短引用读取

```rust
fn read<'a>(slot: &&'a str, local: &'a str) {
    println!("{} / {}", *slot, local);
}

fn main() {
    let text: &'static str = "hello";
    let local = String::from("world");
    read(&text, &local); // 可以
}
```

`read` 需要里面的字符串在 `'a` 期间有效；hello 是静态字符串，当然满足。
两个参数里的 `'a` 表示共同满足的使用范围，不要求两个原始值恰好同时创建和销毁。

```text
& &'static str → & &'a str
  └─────────┘     └─────┘
       T 可以跟着缩短
```

这里只能读取，无法通过 slot 把一个短引用写回 text。因此 `&T` 对 T 协变。

普通打印函数用 `&str` 即可；这里用 `&&str`，只是为了展示外层引用对内层类型的关系。

## 可变引用：不能把长期引用槽位当成短期引用槽位

```rust
fn replace<'a>(slot: &mut &'a str, value: &'a str) {
    *slot = value;
}

fn main() {
    let mut text: &'static str = "hello";
    {
        let local = String::from("world");
        replace(&mut text, &local); // 拒绝：local 不满足 'static
    }
    println!("{text}");
}
```

假如允许：

```text
text 声明只能保存 'static 引用
          │
          │ 假如 &mut &'static str 能当成 &mut &'a str
          v
replace 把 &local 写进 text
          │
          v
local 被销毁，text 却还存在
          │
          v
再次读取 text，引用已经悬空
```

因此 `&mut T` 对 T 不变。反方向也不成立：短期有效的引用不能当作长期引用读取。

| 类型 | 改变的位置 | 结果 |
|---|---|---|
| `&'a T` | 外层 `'a` | 协变，可以缩短外层借用时间 |
| `&'a T` | 内层 T | 协变，允许 T 的子类型替换 |
| `&'a mut T` | 外层 `'a` | 协变，可以缩短外层借用时间 |
| `&'a mut T` | 内层 T | 不变，防止写回不符合原约束的值 |

## 自查与关联

为什么缩短借用时间不等于允许写入任意短引用？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
