# Weak 与强引用环

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**Weak 不保证对象存活，upgrade 返回 Option；反向关系用 Weak 可避免强引用环。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/worker_registry.rs](../../../../src/worker_registry.rs)：`workers: HashMap<String, Arc<Worker>>` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `Arc` 与 `Weak` 的区别

| 类型 | 是否拥有对象 | 是否增加强引用计数 | 能否保证对象存活 |
|---|---:|---:|---:|
| `Arc<T>` | 是 | 是 | 是 |
| `Weak<T>` | 否 | 否 | 否 |

可以把强引用计数理解成对象的“存活票数”：

```text
strong_count > 0  -> T 一定还活着
strong_count == 0 -> T 被销毁
```

`Weak<T>` 不会让 `strong_count` 增加，所以它不能单独让 `T` 继续存活。

## 为什么两个 `Arc` 会泄漏

假设父子双方都用 `Arc` 保存对方：

```text
外部 ──Arc──> Parent ──Arc──> Child
                  ▲                 │
                  └─────Arc─────────┘
```

当外部引用释放后：

```text
Parent strong_count = 1  （Child 持有）
Child  strong_count = 1  （Parent 持有）
```

两个对象都在等待对方先释放，但谁也不会先释放：

```text
Parent 释放条件：strong_count == 0  ✗
Child  释放条件：strong_count == 0  ✗
```

结果是内存泄漏。

## 用 `Weak` 断开“反向指针”

通常让“拥有关系”保持 `Arc`，让“回指 / 父指针 / 注册表反查”变成 `Weak`：

```text
外部 ──Arc──> Parent ──Arc──> Child
                  ▲                 │
                  └────Weak─────────┘
```

外部释放时，释放顺序变成：

```text
1. 外部 Arc<Parent> 被 drop
2. Parent strong_count = 0 -> Parent 被释放
3. Parent 持有的 Arc<Child> 被 drop
4. Child strong_count = 0 -> Child 被释放
5. Child 中的 Weak<Parent> 自动失效
```

关键不是“`Weak` 自动清理循环”，而是：

```text
Weak 不算拥有者
=> 它不会阻止 strong_count 降到 0
=> 对象可以析构
```

## `upgrade()`：弱引用不能直接使用

`Weak<T>` 指向的对象可能已经释放。使用前必须升级：

```rust
if let Some(context) = weak_context.upgrade() {
    // context: Arc<AppContext>
    // AppContext 仍存活；在此作用域内可安全使用。
} else {
    // 没有任何 Arc<AppContext> 了，对象已经释放。
}
```

类型变化：

```text
Weak<AppContext>
       │ .upgrade()
       ▼
Option<Arc<AppContext>>
       │
       ├─ Some(Arc<_>)：对象存活
       └─ None        ：对象已释放
```

不能把 `Weak` 当作永远有效的 `Arc`：

```rust
// ❌ weak_context.do_something()

// ✅ 先升级
let context = weak_context.upgrade()?;
context.do_something();
```

## 自查与关联

弱引用还在时数据一定在吗？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [RefCell：单线程里的运行时借用检查](refcell.md)
