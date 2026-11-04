//! 启动配置解析、业务校验和分层错误。

use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::path::Path;
use url::Url;

/// JSON 中以 kebab-case 表示的策略名称。
// 把 JSON 的 "round-robin" 映射到 RoundRobin。
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyKind {
    FirstHealthy,
    RoundRobin,
}

/// 节点注册信息；地址必须是可解析的 HTTP 或 HTTPS URL。
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WorkerConfig {
    pub id: String,
    pub address: String,
}

/// 启动配置；解析和业务校验是两个独立步骤。
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct GatewayConfig {
    pub workers: Vec<WorkerConfig>,
    pub policy: PolicyKind,
}

/// 区分文件读取、JSON 解析和业务校验失败，保留底层错误链。
#[derive(Debug)]
pub enum ConfigLoadError {
    Read(std::io::Error),
    Parse(serde_json::Error),
    //这里相当于是包了一层,错误在结构上做了分层
    Validation(ConfigError),
}

impl From<std::io::Error> for ConfigLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Read(error)
    }
}

impl From<serde_json::Error> for ConfigLoadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(error)
    }
}

impl From<ConfigError> for ConfigLoadError {
    fn from(error: ConfigError) -> Self {
        Self::Validation(error)
    }
}

impl std::error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => {
                write!(f, "failed to read config: {error}")
            }
            Self::Parse(error) => {
                write!(f, "failed to parse config: {error}")
            }
            Self::Validation(error) => {
                write!(f, "invalid config: {error}")
            }
        }
    }
}

/// 配置业务约束失败，携带用于定位问题的节点信息。
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    EmptyWorkers,
    DuplicateWorkerId(String),
    InvalidWorkerAddress { worker_id: String, address: String },
}

impl Display for ConfigError {
    //&mut的原因是写入输出会改变缓冲区状态, '_这个符合表示生命周期由编译器推断
    //<'_> 内部借用了输出缓冲区，因此需要生命周期
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWorkers => {
                write!(f, "at least one worker is required")
            }
            Self::DuplicateWorkerId(worker_id) => {
                write!(f, "duplicate worker id: {worker_id}")
            }
            Self::InvalidWorkerAddress { worker_id, address } => {
                write!(f, "invalid address for worker {worker_id}: {address}")
            }
        }
    }
}

//ConfigError 是标准错误,但目前没有额外暴露底层 source
impl std::error::Error for ConfigError {}

impl GatewayConfig {
    /// 读取并校验配置；文件、JSON 和业务错误分别返回对应变体。
    pub fn load(path: &Path) -> Result<Self, ConfigLoadError> {
        let input = std::fs::read_to_string(path)?;
        Self::parse_json_validated(&input)
    }

    /// 仅解析 JSON，不保证业务约束有效。
    pub fn parse_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
        //? 表示   成功时取出 Ok 里面的值；  失败时让当前函数立即返回 Err。
    }

    /// 解析并校验 JSON；成功结果满足 `validate` 的约束。
    pub fn parse_json_validated(input: &str) -> Result<Self, ConfigLoadError> {
        let config = Self::parse_json(input)?;
        config.validate()?;
        Ok(config)
    }

    //这里的(),是unit类型，表示成功了，但没有额外数据需要返回
    /// 校验非空节点池、唯一 ID 和 HTTP(S) 地址；返回首个错误。
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.workers.is_empty() {
            //使用return提前返回
            return Err(ConfigError::EmptyWorkers);
        }

        let mut worker_uniques: HashSet<&str> = HashSet::new();
        for worker in &self.workers {
            if !worker_uniques.insert(worker.id.as_str()) {
                return Err(ConfigError::DuplicateWorkerId(worker.id.clone()));
            }

            let parsed_url = match Url::parse(worker.address.as_str()) {
                Ok(url) => url,
                Err(_) => {
                    return Err(ConfigError::InvalidWorkerAddress {
                        worker_id: worker.id.clone(),
                        address: worker.address.clone(),
                    });
                }
            };

            if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
                return Err(ConfigError::InvalidWorkerAddress {
                    worker_id: worker.id.clone(),
                    address: worker.address.clone(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, ConfigLoadError, GatewayConfig, PolicyKind};
    use std::path::Path;

    #[test]
    fn loads_valid_config_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/example.json");
        let config = GatewayConfig::load(&path).expect("example config should be valid");

        assert_eq!(config.workers.len(), 2)
    }

    #[test]
    fn reports_missing_config_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/does-not-exist.json");

        let result = GatewayConfig::load(&path);

        assert!(matches!(result, Err(ConfigLoadError::Read(_))));
    }

    // 保留显式 JSON：测试同时覆盖反序列化与业务校验，不用构造 struct 绕过解析。
    #[test]
    fn validated_json_distinguishes_error_stages() {
        let valid = r#"
        {
            "workers": [
                {
                    "id": "worker-a",
                    "address": "https://127.0.0.1:8001"
                }
            ],
            "policy": "round-robin"
        }
        "#;

        assert!(GatewayConfig::parse_json_validated(valid).is_ok());

        let malformed = "{";

        assert!(matches!(
            GatewayConfig::parse_json_validated(malformed),
            Err(ConfigLoadError::Parse(_))
        ));

        let empty_workers = r#"
        {
            "workers": [],
            "policy": "round-robin"
        }
        "#;

        assert!(matches!(
            GatewayConfig::parse_json_validated(empty_workers),
            Err(ConfigLoadError::Validation(ConfigError::EmptyWorkers))
        ));
    }

    #[test]
    fn config_errors_have_readable_messages() {
        assert_eq!(
            ConfigError::EmptyWorkers.to_string(),
            "at least one worker is required"
        );

        assert_eq!(
            ConfigError::DuplicateWorkerId("worker-a".to_string()).to_string(),
            "duplicate worker id: worker-a"
        );

        assert_eq!(
            ConfigError::InvalidWorkerAddress {
                worker_id: "worker-a".to_string(),
                address: "not-a-url".to_string()
            }
            .to_string(),
            "invalid address for worker worker-a: not-a-url"
        );
    }

    #[test]
    fn rejects_non_http_worker_address() {
        let input = r#"
        {
            "workers": [
                {
                    "id": "worker-a",
                    "address": "ftp://example.com/model"
                }
            ],
            "policy": "round-robin"
        }
        "#;

        let config =
            GatewayConfig::parse_json(input).expect("JSON should parse before address validation");
        assert_eq!(
            Err(ConfigError::InvalidWorkerAddress {
                worker_id: "worker-a".to_string(),
                address: "ftp://example.com/model".to_string()
            }),
            config.validate()
        )
    }

    #[test]
    fn rejects_invalid_worker_address() {
        let input = r#"
        {
            "workers": [
                {
                    "id": "worker-a",
                    "address": "not-a-url"
                }
            ],
            "policy": "round-robin"
        }
        "#;
        let config =
            GatewayConfig::parse_json(input).expect("JSON should parse before address validation");
        assert_eq!(
            Err(ConfigError::InvalidWorkerAddress {
                worker_id: "worker-a".to_string(),
                address: "not-a-url".to_string()
            }),
            config.validate()
        )
    }

    #[test]
    fn rejects_duplicate_worker_ids() {
        let input = r#"
        {
            "workers": [
                {
                    "id": "worker-a",
                    "address": "http://127.0.0.1:8001"
                },
                {
                    "id": "worker-a",
                    "address": "http://127.0.0.1:8002"
                }
            ],
            "policy": "round-robin"
        }
        "#;

        let config = GatewayConfig::parse_json(input).expect("valid config should parse");
        assert_eq!(
            Err(ConfigError::DuplicateWorkerId("worker-a".to_string())),
            config.validate()
        );
    }

    #[test]
    fn rejects_malformed_json() {
        let input = r#"
        {"sdsds":"sdsdsd"
        "#;
        let config_result = GatewayConfig::parse_json(input);
        assert!(config_result.is_err());
    }

    #[test]
    fn parses_empty_workers_but_validation_rejects_them() {
        let input = r#"
        {
            "workers": [],
            "policy": "round-robin"
        }
        "#;

        let config = GatewayConfig::parse_json(input).expect("valid config should parse");
        assert_eq!(Err(ConfigError::EmptyWorkers), config.validate());
    }

    #[test]
    fn parses_valid_json_config() {
        let input = r#"
        {
            "workers": [
                {
                    "id": "worker-a",
                    "address": "http://127.0.0.1:8001"
                },
                {
                    "id": "worker-b",
                    "address": "http://127.0.0.1:8002"
                }
            ],
            "policy": "round-robin"
        }
        "#;

        let config = GatewayConfig::parse_json(input).expect("valid config should parse");

        assert_eq!(Ok(()), config.validate());
        assert_eq!(config.workers.len(), 2);
        assert_eq!(config.workers[0].id, "worker-a");
        assert_eq!(config.workers[1].address, "http://127.0.0.1:8002");
        assert_eq!(config.policy, PolicyKind::RoundRobin);
    }
}
