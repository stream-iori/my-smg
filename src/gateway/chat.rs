//! 普通响应转发：完整读取正文后提交熔断结果，总预算覆盖重试和退避。
use super::gateway_state::{ChatRequest, GatewayState};
use super::retry;
use crate::RequestId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use my_smg::circuit_gate::CircuitGateError;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) fn circuit_gate_error_response(error: CircuitGateError) -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("circuit gate unavailable: {error}"),
    )
}

pub(super) fn upstream_error_response(error: &reqwest::Error, phase: &str) -> (StatusCode, String) {
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

pub(crate) async fn chat(
    State(state): State<Arc<GatewayState>>,
    request_id: Option<Extension<RequestId>>,
    Json(request): Json<ChatRequest>,
) -> (StatusCode, String) {
    let _concurrency_permit = match state.acquire_request_permit().await {
        Ok(permit) => permit,
        Err(error) => return error,
    };
    let worker = match state.select_worker() {
        Ok(worker) => worker,
        Err(error) => return error,
    };

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
    let result =
        tokio::time::timeout(Duration::from_secs(2), retry::retry_with_backoff(operation)).await;

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
