//! 教学模型网关的节点、路由、配置和故障控制组件。
//!
//! 请求计数通过凭证的生命周期管理：
//! ```
//! use my_smg::worker::Worker;
//! let worker = Worker::new("a".to_string(), "http://localhost:8000".to_string());
//! let guard = worker.begin_request();
//! assert_eq!(worker.counter(), 1);
//! drop(guard);
//! assert_eq!(worker.counter(), 0);
//! ```

//declare a module and content of module from src/worker.rs
pub mod worker;

pub mod policy;

pub mod circuit_breaker;
pub mod circuit_gate;
pub mod gateway_config;
pub mod token_bucket;
pub mod worker_registry;

mod project_name;
pub use project_name::project_name;

//只在运行测试时编译下面的模块
#[cfg(test)]
mod tests {
    //使用外部函数，模块tests的上一级函数名字
    use super::project_name;

    #[test]
    fn returns_project_name() {
        assert_eq!(project_name(), "my-smg");
    }
}
