use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::path::Path;
use url::Url;

#[derive(Debug, Deserialize, PartialEq, Eq)]
// 把 JSON 的 "round-robin" 映射到RoundRobin
#[serde(rename_all = "kebab-case")]
pub enum PolicyKind {
    FirstHealthy,
    RoundRobin,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WorkerConfig {
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct GatewayConfig {
    pub workers: Vec<WorkerConfig>,
    pub policy: PolicyKind,
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    EmptyWorkers,
    DuplicateWorkerId(String),
    InvalidWorkerAddress(String, String),
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
            Self::InvalidWorkerAddress(worker_id, address) => {
                write!(f, "invalid address for worker {worker_id}: {address}")
            }
        }
    }
}

//ConfigError 是标准错误,但目前没有额外暴露底层 source
impl std::error::Error for ConfigError {}

impl GatewayConfig {
    pub fn from_file(path: &Path) -> Result<Self, ConfigLoadError> {
        let input = std::fs::read_to_string(path)?;
        Self::from_json_validated(&input)
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
        //? 表示   成功时取出 Ok 里面的值；  失败时让当前函数立即返回 Err。
        //let config = serde_json::from_str(input)?;
        //Ok(config)
    }

    pub fn from_json_validated(input: &str) -> Result<Self, ConfigLoadError> {
        let config = Self::from_json(input)?;
        config.validate()?;
        Ok(config)
    }

    //这里的(),是unit类型，表示成功了，但没有额外数据需要返回
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
                    return Err(ConfigError::InvalidWorkerAddress(
                        worker.id.clone(),
                        worker.address.clone(),
                    ));
                }
            };

            if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
                return Err(ConfigError::InvalidWorkerAddress(
                    worker.id.clone(),
                    worker.address.clone(),
                ));
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
        let config = GatewayConfig::from_file(&path).expect("example config should be valid");

        assert_eq!(config.workers.len(), 2)
    }

    #[test]
    fn reports_missing_config_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/does-not-exist.json");

        let result = GatewayConfig::from_file(&path);

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

        assert!(GatewayConfig::from_json_validated(valid).is_ok());

        let malformed = "{";

        assert!(matches!(
            GatewayConfig::from_json_validated(malformed),
            Err(ConfigLoadError::Parse(_))
        ));

        let empty_workers = r#"
        {
            "workers": [],
            "policy": "round-robin"
        }
        "#;

        assert!(matches!(
            GatewayConfig::from_json_validated(empty_workers),
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
            ConfigError::InvalidWorkerAddress("worker-a".to_string(), "not-a-url".to_string(),)
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
            GatewayConfig::from_json(input).expect("JSON should parse before address validation");
        assert_eq!(
            Err(ConfigError::InvalidWorkerAddress(
                "worker-a".to_string(),
                "ftp://example.com/model".to_string()
            )),
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
            GatewayConfig::from_json(input).expect("JSON should parse before address validation");
        assert_eq!(
            Err(ConfigError::InvalidWorkerAddress(
                "worker-a".to_string(),
                "not-a-url".to_string()
            )),
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

        let config = GatewayConfig::from_json(input).expect("valid config should parse");
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
        let config_result = GatewayConfig::from_json(input);
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

        let config = GatewayConfig::from_json(input).expect("valid config should parse");
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

        let config = GatewayConfig::from_json(input).expect("valid config should parse");

        assert_eq!(Ok(()), config.validate());
        assert_eq!(config.workers.len(), 2);
        assert_eq!(config.workers[0].id, "worker-a");
        assert_eq!(config.workers[1].address, "http://127.0.0.1:8002");
        assert_eq!(config.policy, PolicyKind::RoundRobin);
    }
}
