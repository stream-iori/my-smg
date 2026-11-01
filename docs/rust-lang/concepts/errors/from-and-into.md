# From 与 Into：为什么有两个转换入口

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**目标端用 From 明确目标，源值用 Into 借上下文确定目标；通常只实现一份 From。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Trait：类型需要提供什么能力](../types/traits-and-bounds.md) |
| 源码入口 | [src/config.rs](../../../../src/config.rs)：`impl From<std::io::Error> for ConfigLoadError` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 类型转换：Into、From、TryInto 与 TryFrom

`.into()` 来自标准库的 `Into` trait：把当前值转换成调用位置需要的目标类型。转换需要对应的
trait 实现；编译器根据变量标注、函数参数或返回类型推断目标。它不是任意类型之间的强制转换。

## 网关启动错误中的 `.into()`

下面是与 `my-smg` 启动错误相关的独立例子：

```rust
fn startup_error() -> Result<(), Box<dyn std::error::Error>> {
    let error = std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "duplicate worker id",
    );

    Err(error.into())
}
```

```text
std::io::Error
       │ .into()
       ▼
Box<dyn std::error::Error>
       │ Err(...)
       ▼
函数的错误返回值
```

函数声明的错误类型决定了转换目标。这里会把具体错误装进 Box，通过 `dyn Error` 统一返回。
底层错误本身仍被保留，不是转换成错误字符串。若没有足够上下文，编译器可能无法推断目标类型。

## 常见场景与类型推断

| 场景 | 转换 | 目标类型来自哪里 |
|---|---|---|
| 创建拥有所有权的字符串 | `&str` → `String` | 变量标注或函数参数 |
| 统一错误返回类型 | 具体错误 → `Box<dyn Error>` | 函数的错误返回类型 |
| 自定义数据转换 | 源类型 → 目标类型 | 对应 trait 实现与调用上下文 |

```rust
let text: String = "hello".into();
```

这里 `String` 指定目标；从 `&str` 创建 String 会分配并复制文本。其他转换是否分配、复制或
只包装原值，要看具体实现，不能把 `.into()` 一概理解为 clone。

## From 与 Into：同方向转换的两个入口

**两个入口是为了方便不同的调用场景，转换规则通常只写一份。** 先看同一件事：把 `&str` 转成 `String`。

```text
"hello"：&str ──转换──> String

String::from("hello")       从目标类型发起
"hello".into()              从源值发起
```

**① 已经知道要转成什么：From 写得直接。**

```rust
let text = String::from("hello");
```

读这一行就知道目标是 String，不需要查看其他上下文。

**② 上下文已经指定目标：Into 少写一次类型。**

```rust
let text: String = "hello".into();
```

左边已经说明需要 String，右边只说“把我转换过去”。

| 写法 | 目标类型在哪里 | 实际方向 |
|---|---|---|
| `String::from("hello")` | 直接写在调用处 | `&str` → `String` |
| `"hello".into()` | 由变量类型、函数参数或返回类型确定 | 本例也是 `&str` → `String` |

如果没有足够的上下文，编译器可能不知道 `.into()` 的目标类型；它不是固定转换为 String。

**③ Into 还能让函数接受多种输入。**

假设一个接口的参数约束如下，这里只展示签名形状：

```rust
fn set_name(name: impl Into<String>)
```

意思是：不限定传来的就是 String，只要求它能转换为 String。函数内部可以用 `name.into()`
得到 String；调用方可以传字符串字面量，也可以传已有的 String。

```text
&str   ──┐
         ├──> set_name 内部调用 name.into() → String
String ──┘
```

| 传入的值 | 转成 String 时发生什么 |
|---|---|
| `&str` | 复制文本，创建拥有内容的 String |
| 已有的 `String` | 转移已有值，不复制字符串内容 |

**两套入口不代表两套实现，也不是互相撤销的操作。**

```text
实现 From<源类型> for 目标类型
                  │
                  └── 标准库自动提供对应的 Into
```

实现 `From<Source> for Target` 后，标准库的通用实现会自动提供 `Into<Target> for Source`。
因此通常优先用 From 定义转换规则；泛型接口只需允许调用 `.into()` 时，可以用 Into 作为约束，
也能接受只实现了 Into 的类型。参见 [标准库 Into](https://doc.rust-lang.org/std/convert/trait.Into.html)。

**定义转换规则通常用 From；调用时，想明确目标就用 `Target::from(...)`，上下文已明确目标就可以用 `.into()`。**

## 自查与关联

into 是强制转换、clone，还是提前返回？

[Option 与 Result：有无与成败](option-and-result.md) · [? 与 ??：成功继续，失败提前返回](question-mark.md) · [TryFrom、TryInto 与可能失败的转换](fallible-conversions.md)
