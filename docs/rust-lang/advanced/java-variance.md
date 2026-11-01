# Java 泛型与 PECS：变型的入门对照

[分类目录](index.md) · [总入口](../index.md) · [问题索引](../questions.md)

**List 本身不变，extends/super 在使用位置表达读写边界。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | Java 泛型基础；不要求先学习 Rust trait 或生命周期。 |
| 源码入口 | [src/policy.rs](../../../src/policy.rs)：`泛型策略与 Java 对照` |
| 阅读时机 | 进阶扩展，主线可以跳过。 |

## Java：先区分猫和猫列表

假设有以下类型；示例中的列表使用 `java.util.List` 和 `java.util.ArrayList`：

```java
class Animal {}
class Cat extends Animal {}
class Dog extends Animal {}
```

```text
Animal
├── Cat
└── Dog

Cat → Animal：需要动物时，可以提供猫。
```

## 不变：List<Cat> 不是 List<Animal>

```java
List<Cat> cats = new ArrayList<>();
List<Animal> animals = cats; // 拒绝
```

假如允许，两个变量就会指向同一个列表：

```text
cats ──────┐
           ├──> 原本只能保存猫的列表
animals ───┘
```

接下来 `animals.add(new Dog())` 就能往猫列表里放狗，而 `cats.get(0)` 仍按猫使用。
因此 Java 的 `List<T>` 对 `T` 不变。它本身的继承关系不受影响，例如 `ArrayList<Cat>` 仍可赋给 `List<Cat>`。

## 协变视角：extends，把列表作为动物来源

```java
List<Cat> cats = new ArrayList<>();
cats.add(new Cat());

List<? extends Animal> source = cats; // 可以
Animal animal = source.get(0);       // 可以

source.add(new Cat()); // 拒绝
source.add(new Dog()); // 拒绝
```

`? extends Animal` 表示：元素类型是某个确定但当前未知的 Animal 子类型，也可能就是 Animal。

| 操作 | 是否允许 | 原因 |
|---|---|---|
| 读取为 `Animal` | 可以 | 无论里面是哪种子类型，都属于动物 |
| 添加一只猫 | 不可以 | 背后可能是 `List<Dog>` |
| 添加一只狗 | 不可以 | 背后可能是 `List<Cat>` |

这里限制的是添加具体动物；`null` 是特殊情况。列表也不是完全只读，例如仍可能支持 `clear()`。

## 逆变视角：super，把列表作为猫的接收处

```java
List<Animal> animals = new ArrayList<>();
animals.add(new Dog());

List<? super Cat> destination = animals; // 可以
destination.add(new Cat());             // 可以

Object value = destination.get(0); // 可以
Cat cat = destination.get(0);      // 拒绝
```

`List<Cat>`、`List<Animal>`、`List<Object>` 都可以赋给 `List<? super Cat>`，它们都能接收猫。
但是里面原有的值未必是猫，所以读取时只能保证是 Object。

方向可用 Cat 和 Animal 两种边界对照：

```text
Cat → Animal

协变：List<? extends Cat> → List<? extends Animal>
逆变：List<? super Animal> → List<? super Cat>
```

## PECS：来源 extends，去向 super

```java
static <T> void copy(
    List<? extends T> source,
    List<? super T> destination
) {
    for (T value : source) {
        destination.add(value);
    }
}
```

```text
source                  destination
读取 T ─────── T ───────> 写入 T
extends                  super
```

Java 的 `List<T>` 本身仍然不变；通配符是在使用位置表达边界。不要把 `? extends` 当成列表实现改变了。

## 自查与关联

为什么 List<Cat> 不能作为 List<Animal>？

[协变与不变：从 Java 泛型到 Rust 引用](variance.md) · [高阶生命周期与函数输入逆变](higher-ranked-lifetimes.md) · [扩展：上游的 Arc 与 Weak 关系](upstream-ownership.md)
