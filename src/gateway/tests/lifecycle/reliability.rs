//! 流式请求与限流、熔断的组合行为。
use super::http;
use super::{request_stream, stream_fixture};
use crate::gateway::chat::chat;
use crate::gateway::gateway_state::ChatRequest;
use crate::gateway::tests::support::{ChatFixtureConfig, chat_fixture};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use futures_util::StreamExt;
use my_smg::worker::Worker;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use my_smg::circuit_breaker::CircuitState;

fn trip_circuit(worker: &Worker, at: Instant) {
    for _ in 0..3 {
        worker
            .circuit_gate()
            .acquire(at)
            .unwrap()
            .unwrap()
            .failure(at)
            .unwrap();
    }
}

#[tokio::test]
async fn normal_and_stream_requests_share_rate_limit() {
    let fixture = chat_fixture(ChatFixtureConfig {
        token_capacity: 1,
        ..Default::default()
    })
    .await;
    let normal = chat(
        State(Arc::clone(&fixture.state)),
        None,
        Json(ChatRequest {
            message: "normal".into(),
        }),
    )
    .await;
    let stream = request_stream(Arc::clone(&fixture.state)).await;
    let count = fixture.request_count.load(Ordering::Relaxed);
    let permits = fixture.state.concurrency.available_permits();
    let waiting = fixture.state.waiting.available_permits();
    fixture.server.stop().await.expect("server should stop");

    assert_eq!(normal.0, StatusCode::OK);
    assert_eq!(stream.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(count, 1, "rejected stream must not reach upstream");
    assert_eq!(fixture.worker.counter(), 0);
    assert_eq!(permits, 2);
    assert_eq!(waiting, 0);
}

#[tokio::test]
async fn open_circuit_rejects_stream_before_contacting_upstream() {
    let fixture = http::http_stream_fixture(true).await;
    trip_circuit(&fixture.worker, Instant::now());
    let response = request_stream(Arc::clone(&fixture.state)).await;
    let status = response.status();
    let count = fixture.upstream.request_count.load(Ordering::Relaxed);
    let permits = fixture.state.concurrency.available_permits();
    let inflight = fixture.worker.counter();
    fixture.stop().await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(count, 0);
    assert_eq!(inflight, 0);
    assert_eq!(permits, 2);
}

#[tokio::test]
async fn cancelling_half_open_stream_reopens_circuit() {
    let (server, worker, state) = stream_fixture().await;
    trip_circuit(&worker, Instant::now() - Duration::from_secs(6));
    let response = request_stream(Arc::clone(&state)).await;
    let status = response.status();
    let state_during = worker.circuit_gate().state();
    drop(response);
    let state_after = worker.circuit_gate().state();
    let permits = state.concurrency.available_permits();
    server.stop().await.expect("server should stop");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(state_during, Ok(CircuitState::HalfOpen));
    assert_eq!(state_after, Ok(CircuitState::Open));
    assert_eq!(worker.counter(), 0);
    assert_eq!(permits, 2);
}

#[tokio::test]
async fn truncated_stream_counts_as_circuit_failure() {
    let mut fixture = http::http_stream_fixture(true).await;
    // 两次已有失败；本次正文截断应触发第三次失败并打开熔断。
    for _ in 0..2 {
        fixture
            .worker
            .circuit_gate()
            .acquire(Instant::now())
            .unwrap()
            .unwrap()
            .failure(Instant::now())
            .unwrap();
    }
    let response = request_stream(Arc::clone(&fixture.state)).await;
    let status = response.status();
    let mut body = response.into_body().into_data_stream();
    let first = body
        .next()
        .await
        .expect("first chunk")
        .expect("readable chunk");
    fixture.upstream.fail.take().unwrap().send(()).unwrap();
    let result = tokio::time::timeout(Duration::from_secs(1), body.next()).await;
    let circuit = fixture.worker.circuit_gate().state();
    let count = fixture.worker.counter();
    let permits = fixture.state.concurrency.available_permits();
    drop(body);
    fixture.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(&first[..], b"data: first\n\n");
    assert!(
        result
            .expect("read should finish")
            .expect("stream error")
            .is_err()
    );
    assert_eq!(circuit, Ok(CircuitState::Open));
    assert_eq!(count, 0);
    assert_eq!(permits, 2);
}
