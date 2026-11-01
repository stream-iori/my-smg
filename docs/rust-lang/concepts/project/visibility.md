# pub 与受限可见性

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**访问一个项要同时检查项的权限与沿途模块；pub(super) 只扩大到直接父模块的子树。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/gateway.rs](../../../../src/gateway.rs)：`pub(super) struct GatewayState` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `pub`：对照 Java 的 `public` 与 `exports` 两层检查

Java 9 中，一个顶层类型声明为 `public`，不代表其他模块一定能使用它：调用方还需要读取目标模块，目标模块也需要向调用方导出该包。[Java 9：模块声明](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html#jls-7.7)

```text
Java：调用方读取目标 module
        → 目标 module 向调用方 exports 对应 package
          → 类型是 public
            → 所访问的成员也可见

Rust：调用方依赖目标 crate
        → 调用方能访问类型的公开路径
          → 类型是 pub
            → 所访问的成员也可见
```

| 场景 | Java 9 | Rust |
|---|---|---|
| 类型公开，但外部路径未开放 | 未导出包中的 `public` 类型，普通外部代码不能直接访问 | 私有模块中的 `pub` 类型，外部不能沿该私有模块路径访问 |
| 类型公开，成员私有 | `public` 类不使字段自动成为 `public` | `pub struct` 不使字段自动成为 `pub` |
| 内部跨包/模块使用 | 未导出包中的 `public` 类型可供同一 JPMS module 内跨包使用 | `pub(crate)` 项可供当前 crate 内使用，仍须满足路径可见性 |

Java 通过 `exports` 控制包边界，Rust 通过模块路径和项的可见性控制边界。Rust 没有额外的 `module-info.java` 来列出所有导出包。

## Rust 可见性表

| 写法 | 可见范围 | Java 对照提醒 |
|---|---|---|
| 无 `pub` | 定义所在模块及其子模块 | 不等同于 Java 默认的包访问权限 |
| `pub(super)` | 父模块及其子模块范围 | 不等同于 Java `protected`，没有继承语义 |
| `pub(in 路径)` | 指定祖先模块及其子模块范围 | 可以精确限定内部协作区域，Java 没有直接对应的修饰符 |
| `pub(crate)` | 当前 crate 内 | 接近模块内部 API 的意图，但 Java 没有对应修饰符 |
| `pub` | 允许公开访问，实际访问还要检查路径 | 类似 `public` 仍要配合边界开放 |

指定可见范围不保证范围内的所有位置都能沿任意路径访问；沿途模块也必须可访问。常见私有项遵循模块范围规则，公开枚举的变体等另有默认规则。[Rust：Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

**Rust 结构体的私有字段是模块范围的私有。** 同一模块里的普通函数也能访问它，不必是该结构体的方法。不能直接套用 Java “只有类内部才能访问 private 字段”的心智模型。

## 多一层模块时：`pub(super)` 与 `pub(in 路径)`

**`super` 只指直接父模块，不会自动开放给祖父模块。** 假设 `reset` 定义在 `internal::xxx::routing` 中：

```text
crate
└── internal                 ✗ 不能直接访问 reset
    └── xxx                  ✓ 能访问 reset
        ├── diagnostics      ✓ 能访问 reset
        └── routing          ✓ 能访问 reset
            └── reset        ← pub(super)，父模块是 xxx
```

`pub(super)` 的范围是整个 `xxx` 子树，不只是 `xxx` 模块本身。`internal` 位于该子树外，即使写完整路径 `xxx::routing::reset`，也不能绕过权限。

如果需要让 `internal` 也能访问，**不必改成完全公开的 `pub`，也不必调整目录结构**，可以指定祖先模块作为边界：

| `reset` 的可见性 | 范围 | `internal` 能直接访问吗 |
|---|---|---|
| 无 `pub` | `routing` 子树 | ✗ |
| `pub(super)` | `xxx` 子树 | ✗ |
| `pub(in crate::internal)` | `internal` 子树 | ✓ |
| `pub(crate)` | 当前 crate | ✓ |
| `pub` | 允许公开访问，实际范围仍受路径限制 | ✓ |

表中假设沿途模块对访问方可见。若只想在 `internal` 内部协作，`pub(in crate::internal)` 比 `pub(crate)` 或 `pub` 更精确：

```text
crate
├── api                      ✗ 在指定范围外
└── internal                 ✓ 指定边界
    ├── monitor              ✓ 在指定范围内
    └── xxx                  ✓ 在指定范围内
        └── routing          ✓ 在指定范围内
            └── reset        ← pub(in crate::internal)
```

还需要检查三个条件：

| 条件 | 本例怎么理解 |
|---|---|
| 路径必须指向定义模块的祖先 | `internal` 是 `routing` 的祖先，可以指定；兄弟模块 `api` 不可以 |
| 沿途模块也必须可见 | 若 `routing` 是 `xxx` 内的私有模块，`internal` 仍不能穿过它访问 `reset`；需要一并开放该模块路径 |
| 看模块树，而非仅看文件夹 | 单纯新建 `xxx` 文件夹不会改变 `super`；模块声明建立父子关系后才会改变 |

在 Rust 2018 及以后，`pub(in 路径)` 的路径须以 `crate`、`self` 或 `super` 开始，并直接解析到祖先模块，不能使用 `use` 引入的别名。[Rust：受限可见性](https://doc.rust-lang.org/reference/visibility-and-privacy.html#pubin-path-pubcrate-pubsuper-and-pubself)

## 用一次访问检查理解全部可见性

再假设 `Gateway` 有公开方法 `status` 和私有字段 `name`；各访问路径没有其他权限障碍。

| 项 | 谁能访问 | 谁不能访问 | 原因 |
|---|---|---|---|
| `Gateway` | 两个 binary 和库内部 | — | 类型公开，且 `api` 路径公开 |
| `Gateway` 的 `status` 方法 | 两个 binary 和库内部 | — | 类型、路径、方法都可见 |
| `Gateway` 的 `name` 字段 | `api` 模块及其子模块 | 库的兄弟模块、两个 binary | 字段私有，边界是定义模块 |
| `normalize` | `internal` 及其子模块，包括 `routing` | `api`、库根、两个 binary | 无 `pub`，仅定义模块及其后代可见 |
| `choose` | 库根、`api`、`internal`、`routing` | 两个 binary | `pub(crate)` 只覆盖 library crate |
| `reset` | `internal` 及其子模块 | `api`、库根、两个 binary | `pub(super)` 将范围扩大到父模块 `internal` |
| `cursor` | `routing` 及其子模块 | `internal`、`api`、库根、两个 binary | 子模块私有项不会反向开放给父模块 |

为什么 `api` 能通过路径访问私有的 `internal` 模块？因为 `internal` 声明在库根，库根及其所有后代都能访问这个模块；进入模块后，里面的函数仍各自检查权限。**模块可见，不代表它的全部内容都可见。**

## 自查与关联

祖父模块能否访问 pub(super) 的函数？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
