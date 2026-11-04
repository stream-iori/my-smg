//! 网关模块入口；实现保留在二进制内部。
mod chat;
mod chat_stream;
mod gateway_state;
mod management;
mod retry;

pub(crate) use chat::chat;
pub(crate) use chat_stream::chat_stream;
pub(crate) use gateway_state::GatewayState;
pub(crate) use management::{add_worker, health, remove_worker, start_health_checks};

#[cfg(test)]
mod tests;
