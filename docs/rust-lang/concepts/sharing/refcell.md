# RefCell：单线程里的运行时借用检查

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**单线程也有所有权和别名问题；RefCell 允许共享入口临时修改，但不允许借用冲突。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [共享借用与可变借用](../ownership/borrowing.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/bin/fake-worker.rs](../../../../src/bin/fake-worker.rs)：`failures_remaining` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## 单线程为什么还需要它

单线程没有多个线程并行执行，但仍可能有多个地方持有同一对象的句柄。Rc 解决存活问题，RefCell 解决“经由共享入口，按需申请修改”的问题。

当前项目没有使用 RefCell；下面是独立对照例子，项目的跨线程共享使用 Mutex 或原子变量。

```rust
use std::cell::RefCell;
use std::rc::Rc;

let a = Rc::new(RefCell::new(0));
let b = Rc::clone(&a);
{
    let mut value = a.borrow_mut();
    *value += 1;
}
assert_eq!(*b.borrow(), 1);
```

```text
a ──Rc──┐
        ├──> RefCell(同一份 0)
b ──Rc──┘         │ borrow_mut
                  ↓
                 变成 1
```

| 操作 | 运行时规则 |
|---|---|
| `borrow()` | 可以有多个共享借用，只要没有可变借用 |
| `borrow_mut()` | 必须没有其他借用 |
| 冲突时 | borrow/borrow_mut 会 panic；try_borrow/try_borrow_mut 返回错误 |

规则没有消失，只是把内部借用检查从编译期移到运行时。守卫结束后才归还相应访问权限。

```text
一个拥有者 + 可以直接提供 &mut T → 优先普通借用
多个拥有者 + 只读取             → Rc<T>
单线程共享 + 临时修改            → Rc<RefCell<T>>
跨线程共享 + 互斥修改            → Arc<Mutex<T>>
```

不能把 Mutex 简单换成 RefCell 以减少开销；RefCell 本身不支持这种跨线程共享。

## 自查与关联

只在单线程运行就能随意同时存在多个 &mut 吗？

[Box：独占拥有堆上对象](box.md) · [Rc 与 Arc：多个句柄共同拥有数据](rc-and-arc.md) · [Weak 与强引用环](weak-and-cycles.md)
