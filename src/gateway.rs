//! 网关运行逻辑：共享状态、HTTP 处理器和后台健康检查。
//! 模块仍属于 binary；不向 library 公开教学内部接口。
mod retry;

use axum::{
    Extension, Json,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use futures_util::stream;
use my_smg::{
    circuit_breaker::gate::CircuitGateError,
    config::{GatewayConfig, PolicyKind, WorkerConfig},
    policy::Policy,
    token_bucket::TokenBucket,
    worker::{HealthStatus, Worker},
    worker_registry::WorkerRegistry,
};
use retry::retry_with_backoff;
use serde::{Deserialize, Serialize};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{Semaphore, TryAcquireError};
use tokio::task::JoinHandle;
use tokio::time::Instant as TokioInstant;

use crate::RequestId;

const STREAM_HEADER_TIMEOUT: Duration = Duration::from_secs(1);
const STREAM_READ_IDLE_TIMEOUT: Duration = Duration::from_secs(2);
const STREAM_TOTAL_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize, Serialize)]
pub(super) struct ChatRequest {
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum StreamOutcome {
    Cancelled,
    Complete,
    ReadError,
    Timeout,
    GateError,
}

struct StreamLogGuard {
    request_id: Option<u64>,
    worker_id: String,
    status: StatusCode,
    started: TokioInstant,
    outcome: StreamOutcome,
}

impl StreamLogGuard {
    fn set_outcome(&mut self, outcome: StreamOutcome) {
        self.outcome = outcome;
    }
}

impl Drop for StreamLogGuard {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed();
        let metric_end = match self.outcome {
            StreamOutcome::Complete => crate::metrics::StreamEnd::Complete,
            StreamOutcome::Cancelled => crate::metrics::StreamEnd::Cancelled,
            StreamOutcome::ReadError | StreamOutcome::Timeout | StreamOutcome::GateError => {
                crate::metrics::StreamEnd::Failed
            }
        };
        crate::metrics::record_stream(metric_end, elapsed);
        let outcome = match self.outcome {
            StreamOutcome::Cancelled => "cancelled",
            StreamOutcome::Complete => "complete",
            StreamOutcome::ReadError => "read_error",
            StreamOutcome::Timeout => "timeout",
            StreamOutcome::GateError => "gate_error",
        };

        tracing::info!(
            event = "stream_body_finished",
            request_id = ?self.request_id,
            worker_id = %self.worker_id,
            status = self.status.as_u16(),
            outcome = outcome,
            body_lifetime_ms = elapsed.as_secs_f64() * 1000.0,
        );
    }
}

// 仅向父模块开放，供 main 组装状态；不是对外的公共库 API。
// 测试是本模块的子模块，因此也能访问这里的内部实现。
pub(super) struct GatewayState {
    pub(super) client: reqwest::Client,
    pub(super) registry: Mutex<WorkerRegistry>,
    pub(super) policy: Mutex<Box<dyn Policy + Send>>,
    pub(super) token_bucket: Mutex<TokenBucket>,
    pub(super) concurrency: Arc<Semaphore>,
    pub(super) waiting: Arc<Semaphore>,
    pub(super) queue_timeout: Duration,
}

fn is_retryable_status(status: StatusCode) -> bool {
    match status {
        StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT | StatusCode::SERVICE_UNAVAILABLE => {
            true
        }
        _ => false,
    }
}

fn circuit_gate_error_response(error: CircuitGateError) -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("circuit gate unavailable: {error}"),
    )
}

fn upstream_error_response(error: &reqwest::Error, phase: &str) -> (StatusCode, String) {
    if error.is_timeout() {
        (
            StatusCode::GATEWAY_TIMEOUT,
            format!("upstream {phase} timed out"),
        )
    } else {
        (
            StatusCode::BAD_GATEWAY,
            format!("upstream {phase} failed: {error}"),
        )
    }
}

pub(super) async fn chat_stream(
    State(state): State<Arc<GatewayState>>,
    request_id: Option<Extension<RequestId>>,
    Json(request): Json<ChatRequest>,
) -> Response {
    let allowed = {
        let mut bucket = match state.token_bucket.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return (StatusCode::SERVICE_UNAVAILABLE, "rate limiter unavailiable")
                    .into_response();
            }
        };
        bucket.try_acquire(Instant::now())
    };

    if !allowed {
        return (StatusCode::TOO_MANY_REQUESTS, "rate limit excceded").into_response();
    }

    let concurrency_permit = match Arc::clone(&state.concurrency).try_acquire_owned() {
        Ok(permit) => permit,

        Err(TryAcquireError::Closed) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "gateway concurrency unavailable",
            )
                .into_response();
        }

        Err(TryAcquireError::NoPermits) => {
            // 等待凭证只覆盖排队，仍然使用借用型。
            let _waiting_permit = match state.waiting.try_acquire() {
                Ok(permit) => permit,

                Err(TryAcquireError::NoPermits) => {
                    return (StatusCode::SERVICE_UNAVAILABLE, "gateway queue full").into_response();
                }

                Err(TryAcquireError::Closed) => {
                    return (StatusCode::SERVICE_UNAVAILABLE, "gateway queue unavailable")
                        .into_response();
                }
            };

            let acquired = tokio::time::timeout(
                state.queue_timeout,
                Arc::clone(&state.concurrency).acquire_owned(),
            )
            .await;

            match acquired {
                Ok(Ok(permit)) => permit,

                Ok(Err(_)) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway concurrency unavailable",
                    )
                        .into_response();
                }

                Err(_) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway queue wait timed out",
                    )
                        .into_response();
                }
            }

            // 离开分支，释放等待凭证。
            // 执行凭证交给外层 concurrency_permit。
        }
    };

    let snapshot = {
        let registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "worker registry unavailiable",
                )
                    .into_response();
            }
        };

        registry.snapshot()
    };

    let selected_index = {
        let mut policy = match state.policy.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return (StatusCode::SERVICE_UNAVAILABLE, "worker policy unavailable")
                    .into_response();
            }
        };
        policy.select(&snapshot)
    };
    let Some(index) = selected_index else {
        return (StatusCode::SERVICE_UNAVAILABLE, "no healthy worker").into_response();
    };

    let worker = &snapshot[index];

    if let Some(Extension(RequestId(id))) = request_id {
        tracing::info!(
            event = "worker_selected",
            request_id = id,
            worker_id = %worker.id(),
        );
    }

    let circuit_permit = match worker.circuit_gate().acquire(Instant::now()) {
        Ok(Some(permit)) => permit,
        Ok(None) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "worker circuit rejected request",
            )
                .into_response();
        }
        Err(error) => {
            return circuit_gate_error_response(error).into_response();
        }
    };

    // 从请求上游之前开始计数。
    let guard = worker.begin_request_owned();

    let url = format!("{}/chat/stream", worker.address().trim_end_matches('/'));

    //总预算从开始请求上游计时，不包含前面的排队
    let started = TokioInstant::now();
    let total_deadline = started + STREAM_TOTAL_TIMEOUT;

    //等待响应头，也不能超过总截止时间
    let header_deadline = (started + STREAM_HEADER_TIMEOUT).min(total_deadline);
    let header_timeout_message = if header_deadline == total_deadline {
        "upstream total deadline exceeded"
    } else {
        "upstream response headers timed out"
    };

    let send_result = tokio::time::timeout_at(
        header_deadline,
        state.client.post(url).json(&request).send(),
    )
    .await;

    let upstream = match send_result {
        Ok(Ok(response)) => response,

        Ok(Err(error)) => {
            if let Err(gate_error) = circuit_permit.failure(Instant::now()) {
                return circuit_gate_error_response(gate_error).into_response();
            }

            return upstream_error_response(&error, "request").into_response();
        }

        Err(_) => {
            if let Err(gate_error) = circuit_permit.failure(Instant::now()) {
                return circuit_gate_error_response(gate_error).into_response();
            }

            return (StatusCode::GATEWAY_TIMEOUT, header_timeout_message).into_response();
        }
    };

    if TokioInstant::now() >= header_deadline {
        if let Err(gate_error) = circuit_permit.failure(Instant::now()) {
            return circuit_gate_error_response(gate_error).into_response();
        }

        return (StatusCode::GATEWAY_TIMEOUT, header_timeout_message).into_response();
    }

    let status = upstream.status();
    let content_type = upstream.headers().get(CONTENT_TYPE).cloned();
    let cache_control = upstream.headers().get(CACHE_CONTROL).cloned();

    let log_guard = StreamLogGuard {
        request_id: request_id.map(|Extension(RequestId(id))| id),
        worker_id: worker.id().to_string(),
        status,
        started: TokioInstant::now(),
        outcome: StreamOutcome::Cancelled,
    };

    let stream = stream::try_unfold(
        (
            upstream,
            guard,
            concurrency_permit,
            circuit_permit,
            total_deadline,
            status,
            log_guard,
        ),
        |(
            mut upstream,
            guard,
            concurrency_permit,
            circuit_permit,
            total_deadline,
            status,
            mut log_guard,
        )| async move {
            let now = TokioInstant::now();

            if now >= total_deadline {
                log_guard.set_outcome(StreamOutcome::Timeout);

                circuit_permit
                    .failure(Instant::now())
                    .map_err(io::Error::other)?;

                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "upstream total deadline exceeded",
                ));
            }

            let idle_deadline = now + STREAM_READ_IDLE_TIMEOUT;
            let read_deadline = idle_deadline.min(total_deadline);

            let timeout_message = if read_deadline == total_deadline {
                "upstream total deadline exceeded"
            } else {
                "upstream body read idle timed out"
            };

            let chunk = match tokio::time::timeout_at(read_deadline, upstream.chunk()).await {
                Ok(Ok(chunk)) => chunk,

                Ok(Err(error)) => {
                    log_guard.set_outcome(StreamOutcome::ReadError);

                    circuit_permit
                        .failure(Instant::now())
                        .map_err(io::Error::other)?;

                    return Err(io::Error::other(error));
                }

                Err(_) => {
                    log_guard.set_outcome(StreamOutcome::Timeout);

                    circuit_permit
                        .failure(Instant::now())
                        .map_err(io::Error::other)?;

                    return Err(io::Error::new(io::ErrorKind::TimedOut, timeout_message));
                }
            };

            if TokioInstant::now() >= read_deadline {
                log_guard.set_outcome(StreamOutcome::Timeout);

                circuit_permit
                    .failure(Instant::now())
                    .map_err(io::Error::other)?;

                return Err(io::Error::new(io::ErrorKind::TimedOut, timeout_message));
            }

            match chunk {
                Some(bytes) => Ok::<_, io::Error>(Some((
                    bytes,
                    (
                        upstream,
                        guard,
                        concurrency_permit,
                        circuit_permit,
                        total_deadline,
                        status,
                        log_guard,
                    ),
                ))),

                None => {
                    // 如果下面更新断路器失败，日志记录 gate_error。
                    log_guard.set_outcome(StreamOutcome::GateError);

                    if status.is_server_error() {
                        circuit_permit
                            .failure(Instant::now())
                            .map_err(io::Error::other)?;
                    } else {
                        circuit_permit
                            .success(Instant::now())
                            .map_err(io::Error::other)?;
                    }

                    log_guard.set_outcome(StreamOutcome::Complete);

                    Ok(None)
                }
            }
        },
    );
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    if let Some(value) = content_type {
        response.headers_mut().insert(CONTENT_TYPE, value);
    }

    if let Some(value) = cache_control {
        response.headers_mut().insert(CACHE_CONTROL, value);
    }

    response
}

pub(super) async fn chat(
    State(state): State<Arc<GatewayState>>,
    request_id: Option<Extension<RequestId>>,
    Json(request): Json<ChatRequest>,
) -> (StatusCode, String) {
    let allowed = {
        let mut bucket = match state.token_bucket.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "rate lmiter unavailable".to_string(),
                );
            }
        };
        bucket.try_acquire(Instant::now())
    };

    // 上面的大括号结束，MutexGuard 已销毁，锁已释放。
    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded".to_string(),
        );
    }

    let _concurrency_permit = match state.concurrency.try_acquire() {
        Ok(permit) => permit,

        Err(TryAcquireError::Closed) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "gateway concurrency unavailable".to_string(),
            );
        }

        Err(TryAcquireError::NoPermits) => {
            // 只有占到等待名额，才允许等待执行名额。
            let _waiting_permit = match state.waiting.try_acquire() {
                Ok(permit) => permit,

                Err(TryAcquireError::NoPermits) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway queue full".to_string(),
                    );
                }

                Err(TryAcquireError::Closed) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway queue unavailable".to_string(),
                    );
                }
            };

            let acquired =
                tokio::time::timeout(state.queue_timeout, state.concurrency.acquire()).await;

            match acquired {
                Ok(Ok(permit)) => permit,

                Ok(Err(_)) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway concurrency unavailable".to_string(),
                    );
                }

                Err(_) => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gateway queue wait timed out".to_string(),
                    );
                }
            }
            // 离开此分支，_waiting_permit 自动释放。
            // 返回的执行凭证交给外层 _concurrency_permit 保存。
        }
    };
    let snapshot = {
        let registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(posioned) => posioned.into_inner(),
        };
        registry.snapshot()
    };

    let selected_idex = {
        let mut policy = match state.policy.lock() {
            Ok(gurad) => gurad,
            Err(posioned) => posioned.into_inner(),
        };
        policy.select(&snapshot)
    };

    let Some(index) = selected_idex else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "no healthy worker".to_string(),
        );
    };

    let worker = &snapshot[index];
    if let Some(Extension(RequestId(id))) = request_id {
        tracing::info!(
            event = "worker_selected",
            request_id = id,
            worker_id = %worker.id(),
        );
    }

    // 一个 guard 对应整次逻辑请求，不是每次重试。
    // 它覆盖请求、读响应体和退避；正常返回、总超时或任务取消时都会 Drop。
    let _inflight_guard = worker.begin_request();
    let url = format!("{}/chat", worker.address().trim_end_matches('/'));

    // 每次操作只复制这些共享引用，不移动原来的对象。
    let client = &state.client;
    let url_ref = url.as_str();
    let request_ref = &request;
    let gate = worker.circuit_gate();

    // 闭包可以重复调用，每次创建一个独立的请求 Future
    // 每次调用闭包，都为本次上游尝试申请一个独立凭证
    let operation = move || async move {
        let permit = match gate.acquire(Instant::now()) {
            Ok(Some(permit)) => permit,
            Ok(None) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "worker circuit rejected request".to_string(),
                ));
            }

            Err(error) => {
                return Err(circuit_gate_error_response(error));
            }
        };

        let upstream = match client
            .post(url_ref)
            .json(request_ref)
            .timeout(Duration::from_secs(1))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                //发送失败，单次请求超时，都计入节点失败.
                //若Gate 更新失败，优先返回Gate内部错误。
                permit
                    .failure(Instant::now())
                    .map_err(circuit_gate_error_response)?;
                return Err(upstream_error_response(&error, "request"));
            }
        };

        let status = upstream.status();

        let body = match upstream.text().await {
            Ok(body) => body,
            Err(error) => {
                permit
                    .failure(Instant::now())
                    .map_err(circuit_gate_error_response)?;

                return Err(upstream_error_response(&error, "response"));
            }
        };

        // 必须等完整正文读完，再报告本次尝试的结果。
        // 熔断判断与重试判断分开：所有5xx都计入熔断失败。
        if status.is_server_error() {
            permit
                .failure(Instant::now())
                .map_err(circuit_gate_error_response)?;
        } else {
            // 非5xx不归咎于节点故障，不表示业务请求一定成功。
            permit
                .success(Instant::now())
                .map_err(circuit_gate_error_response)?;
        }

        Ok((status, body))
    };

    // 包住完整重试过程，包括 HTTP 请求、读正文和退避。
    let result = tokio::time::timeout(Duration::from_secs(2), retry_with_backoff(operation)).await;

    match result {
        // 收到了完整上游响应。
        Ok(Ok(response)) => response,

        // 发送或读取失败：返回已经映射好的错误响应。
        Ok(Err(response)) => response,

        // 整个重试过程超时。
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            "upstream total deadline exceeded".to_string(),
        ),
    }
}

pub(super) async fn health(State(state): State<Arc<GatewayState>>) -> String {
    let snapshot = {
        let registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
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

    output
}

pub(super) async fn add_worker(
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
            Err(poisoned) => poisoned.into_inner(),
        };

        registry.insert(worker)
    };

    if inserted {
        (StatusCode::CREATED, "worker registered".to_string())
    } else {
        (StatusCode::CONFLICT, "worker id already exists".to_string())
    }
}

pub(super) async fn remove_worker(
    State(state): State<Arc<GatewayState>>,
    AxumPath(worker_id): AxumPath<String>,
) -> StatusCode {
    let removed = {
        let mut registry = match state.registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        registry.remove(&worker_id)
    };

    if removed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn check_worker_health(client: &reqwest::Client, worker: &Worker) {
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

pub(super) fn start_health_checks(state: Arc<GatewayState>, pause: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let snapshot = {
                let registry = match state.registry.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
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

// 只在测试构建中加载 src/gateway/tests.rs，正式 binary 不包含测试工具。
#[cfg(test)]
mod tests;
