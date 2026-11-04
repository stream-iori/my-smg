//! 流式转发：凭证随响应正文持有，直到完成、失败或取消。
use super::chat::{circuit_gate_error_response, upstream_error_response};
use super::gateway_state::{ChatRequest, GatewayState};
use crate::RequestId;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use futures_util::stream;
use my_smg::circuit_gate::CircuitPermit;
use my_smg::worker::OwnedInFlightGuard;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Instant as TokioInstant;

pub(super) const STREAM_HEADER_TIMEOUT: Duration = Duration::from_secs(1);
pub(super) const STREAM_READ_IDLE_TIMEOUT: Duration = Duration::from_secs(2);
pub(super) const STREAM_TOTAL_TIMEOUT: Duration = Duration::from_secs(10);

struct StreamState {
    upstream: reqwest::Response,
    guard: OwnedInFlightGuard,
    concurrency_permit: OwnedSemaphorePermit,
    circuit_permit: CircuitPermit,
    total_deadline: TokioInstant,
    status: StatusCode,
    log_guard: StreamLogGuard,
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

pub(crate) async fn chat_stream(
    State(state): State<Arc<GatewayState>>,
    request_id: Option<Extension<RequestId>>,
    Json(request): Json<ChatRequest>,
) -> Response {
    let concurrency_permit = match state.acquire_request_permit().await {
        Ok(permit) => permit,
        Err(error) => return error.into_response(),
    };
    let worker = match state.select_worker() {
        Ok(worker) => worker,
        Err(error) => return error.into_response(),
    };

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
        StreamState {
            upstream,
            guard,
            concurrency_permit,
            circuit_permit,
            total_deadline,
            status,
            log_guard,
        },
        |StreamState {
             mut upstream,
             guard,
             concurrency_permit,
             circuit_permit,
             total_deadline,
             status,
             mut log_guard,
         }| async move {
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
                    StreamState {
                        upstream,
                        guard,
                        concurrency_permit,
                        circuit_permit,
                        total_deadline,
                        status,
                        log_guard,
                    },
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
