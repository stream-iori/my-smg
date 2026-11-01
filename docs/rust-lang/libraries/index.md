# 核心三方库

[返回总入口](../index.md) · [阶段阅读清单](../stage-reading.md)

| 专题 | 解决的问题 |
|---|---|
| [Tokio](tokio/index.md) | 运行时、任务、计时与清理 |
| [Axum](axum/index.md) | 服务端路由、提取器与响应 |
| [Reqwest](reqwest/index.md) | 客户端请求、正文与超时 |
| [Serde 与 serde_json](serde/index.md) | 数据与 JSON 的转换、映射、校验 |
| [url](url/index.md) | 地址文本的解析与校验 |

这些专题只覆盖当前项目的直接依赖。Mio/Netty 用于 Tokio 架构对照；thiserror/clap 留在 [扩展阅读](../advanced/index.md)。
语言的 Future、async/await 与库的 Runtime、spawn 分开讲。
