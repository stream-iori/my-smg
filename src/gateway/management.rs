//! 节点管理和健康探测；毒锁时拒绝访问状态。
use super::gateway_state::GatewayState;
use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use my_smg::gateway_config::{GatewayConfig, PolicyKind, WorkerConfig};
use my_smg::worker::{HealthStatus, Worker};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub(crate) async fn health(
    State(state): State<Arc<GatewayState>>,
) -> Result<String, (StatusCode, String)> {
    let snapshot = {
        let registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "worker registry unavailable".to_string(),
                ));
            }
        };
        registry.snapshot()
    };

    let mut output = format!("ready: {} workers\n", snapshot.len());

    for worker in &snapshot {
        output.push_str(&format!(
            "{}: inflight={} health={}\n",
            worker.id(),
            worker.counter(),
            worker.status(),
        ));
    }

    Ok(output)
}

pub(crate) async fn add_worker(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<WorkerConfig>,
) -> (StatusCode, String) {
    // 临时配置只用于复用校验，不改变网关策略。
    let config = GatewayConfig {
        workers: vec![request],
        policy: PolicyKind::FirstHealthy,
    };

    if let Err(error) = config.validate() {
        return (StatusCode::BAD_REQUEST, format!("invalid worker: {error}"));
    }

    let worker_config = &config.workers[0];

    let worker = Arc::new(Worker::new(
        worker_config.id.clone(),
        worker_config.address.clone(),
    ));

    let inserted = {
        let mut registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "worker registry unavailable".to_string(),
                );
            }
        };

        registry.insert(worker)
    };

    if inserted {
        (StatusCode::CREATED, "worker registered".to_string())
    } else {
        (StatusCode::CONFLICT, "worker id already exists".to_string())
    }
}

pub(crate) async fn remove_worker(
    State(state): State<Arc<GatewayState>>,
    AxumPath(worker_id): AxumPath<String>,
) -> StatusCode {
    let removed = {
        let mut registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
        };

        registry.remove(&worker_id)
    };

    if removed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub(super) async fn check_worker_health(client: &reqwest::Client, worker: &Worker) {
    let url = format!("{}/health", worker.address().trim_end_matches('/'));

    let healthy = match client
        .get(url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    };

    let status = if healthy {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unhealthy
    };

    worker.set_status(status);
}

pub(crate) fn start_health_checks(state: Arc<GatewayState>, pause: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let snapshot = {
                let registry = match state.registry.lock() {
                    Ok(guard) => guard,
                    Err(error) => {
                        tracing::error!(event = "health_checks_stopped", error = %error, "worker registry lock is poisoned");
                        return;
                    }
                };
                registry.snapshot()
            };

            for worker in &snapshot {
                check_worker_health(&state.client, worker).await;
            }

            tokio::time::sleep(pause).await;
        }
    })
}
