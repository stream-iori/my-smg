# pub use：内部组织与公开入口

[分类目录](index.md) · [总入口](../../index.md) · [问题索引](../../questions.md)

**重新导出为已有项提供另一个入口，不移动文件，也不创建新类型。**

| 阅读准备 | 内容 |
|---|---|
| 前置 | [crate、self、super 与 use](paths-and-imports.md) · [pub 与受限可见性](visibility.md)基础部分；项目应用另按节标题补前置。 |
| 源码入口 | [src/lib.rs](../../../../src/lib.rs)：`pub mod worker` |
| 阅读时机 | 按当前课程需要阅读，不据此跳过 learning 的阶段验收。 |

## `pub use`：提供公开入口，Java exports 不会改路径

假设 `User` 在内部 `user` 模块中定义，并声明为 `pub`。在 crate 根模块用 `pub use` 重新导出它，可以提供另一条路径：

```text
内部组织路径：my_app::user::User
                    │
                 pub use
                    ↓
对外入口路径：my_app::User
```

| 操作 | 效果 |
|---|---|
| Java `exports com.example.user` | 向外开放该包，类型仍叫 `com.example.user.User` |
| Rust `pub use` 重新导出 `User` | 建立另一个访问路径，例如 `my_app::User` |
| Java 普通 `import` | 只简化当前文件中的名字，不建立对外入口 |

**内部 `user` 模块可以保持私有，同时通过根模块公开重新导出其中的公开 `User`。** 外部只能使用开放的入口，不能因此访问内部模块的其他内容。重新导出不会创建新类型或包装对象。[Rust：Re-exporting and Visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html#re-exporting-and-visibility)

所以，“沿途父模块也要公开”是外部沿原模块路径访问时的要求；`pub use` 可以提供另一条公开路径。

## 文件布局、模块路径与公开入口可以分别设计

**Rust 更明显的优势是：内部文件布局、模块组织、对外 API 路径可以分别设计。** 假设 `Router` 是公开类型，并在 crate 根通过 `pub use` 重新导出：

```text
物理文件：src/internal/xxx/routing.rs
模块路径：crate::internal::xxx::routing::Router
公开入口：gateway_demo::Router
                           ↑
                        pub use
```

这里对外路径变短了，内部目录并没有变浅。`pub use` 隐藏了调用者不需要了解的组织层级；内部仍按职责划分模块，对外可以提供简短、稳定的入口。

| 层面 | 决定什么 | 本例 |
|---|---|---|
| 物理文件布局 | 源码存放在哪里 | `src/internal/xxx/routing.rs` |
| 模块组织 | 类型在模块树中的位置与可见性边界 | `crate::internal::xxx::routing::Router` |
| 对外 API 路径 | 调用者通过什么入口使用类型 | `gateway_demo::Router` |

默认的文件模块布局仍映射模块路径；内联模块和 `#[path]` 可以让模块组织与文件位置有所不同。`pub use` 改变的是公开入口，不会移动文件，也不会改变原模块的父子关系。[Rust：模块与文件位置](https://doc.rust-lang.org/reference/items/modules.html)、[重新导出](https://doc.rust-lang.org/reference/visibility-and-privacy.html#re-exporting-and-visibility)

Java 常用的 `com.company.product` 前缀用于避免包名冲突；Rust 的外部路径从 crate 名开始，通常不需要同样的反向域名前缀。这也会让 Rust 工程看起来更浅。Java 的包命名约定见 [Java 9 语言规范](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html#jls-7.4.1)。

**路径较短不代表目录必须较浅。** Rust 没有“三层以内”的语言限制；目录深度应由实际职责决定。

## Java 9 另外三个关键词不要混淆

| Java 9 写法 | 含义 | 与 Rust 的区别 |
|---|---|---|
| `requires transitive` | 让读取当前模块的其他模块也能读取指定依赖模块 | 不是 `pub use`，不会给类型建立新路径 |
| `exports ... to ...` | 只向指定模块导出包 | `pub(crate)`、`pub(super)` 限制的是内部范围，不能指定某个外部 crate |
| `opens` | 控制包对反射的开放，支持深度反射访问 | 不是普通代码的导出权限；Rust 的 `pub` 没有对应的反射开放语义 |

上述机制见 [Java 9：依赖](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html#jls-7.7.1)与[导出和开放包](https://docs.oracle.com/javase/specs/jls/se9/html/jls-7.html#jls-7.7.2)。

## 改善对外入口：`pub use`

假设库根公开重新导出 `api` 中的 `Gateway`：

```text
原路径：gateway_demo::api::Gateway
                     │
                  pub use
                     ↓
新入口：gateway_demo::Gateway
```

| 场景 | 结果 |
|---|---|
| `api` 保持公开 | 原路径和新入口都能使用 |
| `api` 改为私有，但保留根上的公开重新导出 | 外部使用新入口，不能沿原 `api` 路径访问 |
| 使用新入口访问 `Gateway` 的私有 `name` 字段 | 仍然不允许，重新导出不改变字段权限 |
| 尝试公开重新导出仅为 `pub(crate)` 的 `choose` | 不允许把受限项提升为对外公开项 |

重新导出的是同一个类型，没有创建另一份 `Gateway`。它让调用方不必依赖内部目录组织。[Rust：可见性与重新导出](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

## 自查与关联

公开入口变短是否意味着目录变浅？

[Package 与 Crate：一个工程里有几棵编译树](package-and-crate.md) · [模块声明如何找到文件](modules-and-files.md) · [crate、self、super 与 use](paths-and-imports.md)
