# 迭代时借用还是移动

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**iter 借用元素，into_iter 按接收者类型决定如何迭代；collect 需要确定目标容器。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [数组、切片、Vec 与 usize](arrays-slices-and-vec.md) · [共享借用与可变借用](../ownership/borrowing.md) |
| 源码入口 | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`WorkerRegistry::snapshot` |
| 阅读时机 | T03 读到基础自查；收集和快照按后文阶段阅读。 |

## 容器迭代时的所有权

下例和表格限定接收者为拥有型 Vec<T>，分别对应读取、修改和消费：

```rust
let mut names = vec![
    String::from("alice"),
    String::from("bob"),
];

for name in names.iter() {
    println!("{name}");       // name: &String
}

for name in names.iter_mut() {
    name.make_ascii_uppercase(); // name: &mut String
}

for name in names.into_iter() {
    println!("{name}");       // name: String
}

// names 已被 into_iter() 消费，不能再使用
```

| 写法 | 元素类型 | 容器之后能否使用 |
|---|---|---:|
| `iter()` | `&T` | 能 |
| `iter_mut()` | `&mut T` | 能，借用结束后 |
| `into_iter()` | `T` | 不能，容器已被消费 |

对 &Vec<T> 调用 into_iter 得到元素的共享借用，不会消费原 Vec。相同方法名的行为要结合接收者类型判断。

`for item in collection` 通常调用 `IntoIterator::into_iter(collection)`，也会消费拥有所有权的
容器；不想消费时写 `for item in &collection`。

## 基础自查

| 类型 | 问题 |
|---|---|
| 预测 | 三轮迭代分别取得 &String、&mut String 还是 String？ |
| 解释 | 为什么最后一轮之后不能继续使用 names？ |
| 边界 | 对 &Vec<T> 调用 into_iter，也会取走原 Vec 吗？ |

先写下判断和原因，再用自己的已有代码验证。

[struct、enum、impl 与 Self](struct-enum-and-impl.md) · [元组、元组结构体与 unit](tuples-and-unit.md) · [表达式、返回值与分号](expressions-and-semicolons.md)


## 阶段 3 后：收集迭代结果

collect 消费迭代器，目标容器由上下文或注解确定，见[类型推断](../types/type-inference.md)。map 描述逐项转换，消费迭代器时才执行转换；[闭包](../types/closures-and-fn.md)语法按需再学。

## 阶段 8：注册表快照

先读[Arc](../sharing/rc-and-arc.md)和[闭包](../types/closures-and-fn.md)。values 提供值的迭代，map(Arc::clone) 克隆句柄，collect 创建快照容器。综合题：它复制的是 Worker 内容还是共享句柄？
