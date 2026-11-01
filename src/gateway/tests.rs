//! 测试入口：各子模块按行为分组，support 只提供准备与清理工具。
//! 仍是单元测试，不需要为测试把网关处理器改成公共库 API。
//! rstest 只用于参数化；环境准备显式调用普通函数，场景编排与清理留在用例中。

// 导入父模块的网关类型和函数，供本模块及其测试子模块使用。
use super::*;
use axum::{
    Json, Router,
    routing::{get, post},
};
use my_smg::policy::RoundRobin;
use std::sync::atomic::Ordering;

mod support;
// support 的 pub(super) 项仅供 tests 及其子模块使用，不暴露到生产入口。
use support::*;

mod concurrency;
mod forwarding;
mod health;
mod lifecycle;
mod rate_limit;
mod request_id;
mod workers;
