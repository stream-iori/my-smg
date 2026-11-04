//! 程序入口：读取配置、组装网关状态和路由、启动服务。
//! 请求处理与测试放在 gateway 模块，本文件不展开转发细节。

mod gateway;
mod metrics;
mod shutdown;
mod write_timeout_listener;

use write_timeout_listener::WriteTimeoutListener;

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use gateway::{
    GatewayState, add_worker, chat, chat_stream, health, remove_worker, start_health_checks,
};
use my_smg::gateway_config::{GatewayConfig, PolicyKind};
use my_smg::policy::{FirstHealthy, Policy, RoundRobin};
use my_smg::token_bucket::TokenBucket;
use my_smg::worker::Worker;
use my_smg::worker_registry::WorkerRegistry;
use std::path::Path;
use std::process;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestId(pub(crate) u64);

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);
static REQUESTS_RECEIVED_TOTAL: AtomicU64 = AtomicU64::new(0);

#[derive(serde::Serialize)]
struct MetricSnapshot {
    requests_received_total: u64,
    #[serde(flatten)]
    lifecycle: metrics::MetricsSnapshot,
}

async fn metrics() -> Json<MetricSnapshot> {
    Json(MetricSnapshot {
        requests_received_total: REQUESTS_RECEIVED_TOTAL.load(std::sync::atomic::Ordering::Relaxed),
        lifecycle: metrics::snapshot(),
    })
}

async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let started = Instant::now();
    let request_id = NEXT_REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let track_metrics = path != "/metrics";
    if track_metrics {
        REQUESTS_RECEIVED_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    tracing::info!(
        event = "request_started",
        request_id = request_id,
        method = %method,
        path = %path,
    );

    request.extensions_mut().insert(RequestId(request_id));

    let mut response = next.run(request).await;
    let elapsed = started.elapsed();
    if track_metrics {
        metrics::record_response(response.status(), elapsed);
    }

    let header_value = HeaderValue::from_str(&request_id.to_string())
        .expect("a numeric request ID must be a valid header value");

    response.headers_mut().insert("x-request-id", header_value);

    tracing::info!(
        event = "response_ready",
        request_id = request_id,
        status = response.status().as_u16(),
        response_ready_duration_ms = elapsed.as_secs_f64() * 1000.0,
    );

    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //整个进程只初始化一次
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let Some(config_path) = std::env::args().nth(1) else {
        tracing::error!(
            event = "missing_config_path",
            usage = "my-smg <config-path>",
        );
        process::exit(2);
    };

    let config = match GatewayConfig::load(Path::new(&config_path)) {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(
                event = "config_load_failed",
                config_path = %config_path,
                error = %error,
            );
            process::exit(1);
        }
    };

    let mut registry = WorkerRegistry::new();
    for worker_config in &config.workers {
        let worker = Arc::new(Worker::new(
            worker_config.id.clone(),
            worker_config.address.clone(),
        ));

        if !registry.insert(worker) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("duplicate worker id: {}", worker_config.id),
            )
            .into());
        }
    }

    let policy: Box<dyn Policy + Send> = match &config.policy {
        PolicyKind::FirstHealthy => Box::new(FirstHealthy::new()),
        PolicyKind::RoundRobin => Box::new(RoundRobin::new()),
    };

    let token_bucket = TokenBucket::try_new(2, Duration::from_secs(1), Instant::now())?;

    let state = Arc::new(GatewayState {
        client: reqwest::Client::builder().build()?,
        registry: Mutex::new(registry),
        policy: Mutex::new(policy),
        token_bucket: Mutex::new(token_bucket),
        concurrency: Arc::new(Semaphore::new(2)),
        waiting: Arc::new(Semaphore::new(2)),
        queue_timeout: Duration::from_millis(300),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .route("/chat/stream", post(chat_stream))
        .route("/workers", post(add_worker))
        .route("/workers/{worker_id}", delete(remove_worker))
        .route("/metrics", get(metrics))
        .with_state(state.clone())
        .layer(middleware::from_fn(request_id_middleware));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!(
        event = "gateway_started",
        address = %listener.local_addr()?,
    );

    let health_task = start_health_checks(state.clone(), Duration::from_secs(1));
    let listener = WriteTimeoutListener::new(listener, Duration::from_secs(3));
    shutdown::run(listener, app, health_task).await
}
