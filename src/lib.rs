//declare a module and content of module from src/worker.rs
pub mod worker;

pub mod policy;

pub mod circuit_breaker;
pub mod config;
pub mod token_bucket;
pub mod worker_registry;
pub mod write_timeout;

pub fn project_name() -> &'static str {
    "my-smg"
}

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
