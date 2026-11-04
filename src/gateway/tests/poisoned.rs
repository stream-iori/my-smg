//! 毒锁必须阻止继续读取共享状态，不能触达上游或遗留执行名额。
use crate::gateway::chat::chat;
use crate::gateway::chat_stream::chat_stream;
use crate::gateway::gateway_state::ChatRequest;
use crate::gateway::management;
use crate::gateway::tests::support::{ChatFixtureConfig, chat_fixture};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use my_smg::gateway_config::WorkerConfig;
use rstest::rstest;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy)]
enum PoisonTarget {
    Registry,
    Policy,
    RateLimiter,
}

fn poison<T: Send>(lock: &Mutex<T>) {
    std::thread::scope(|scope| {
        let result = scope
            .spawn(|| {
                let _guard = lock.lock().expect("lock starts usable");
                panic!("intentional poison for rejection test");
            })
            .join();
        assert!(result.is_err());
    });
    assert!(lock.is_poisoned());
}

#[rstest]
#[case(PoisonTarget::Registry)]
#[case(PoisonTarget::Policy)]
#[case(PoisonTarget::RateLimiter)]
#[tokio::test]
async fn requests_reject_poison_without_contacting_upstream(#[case] target: PoisonTarget) {
    let fixture = chat_fixture(ChatFixtureConfig::default()).await;
    match target {
        PoisonTarget::Registry => poison(&fixture.state.registry),
        PoisonTarget::Policy => poison(&fixture.state.policy),
        PoisonTarget::RateLimiter => poison(&fixture.state.token_bucket),
    }
    let request = || {
        Json(ChatRequest {
            message: "poisoned".to_string(),
        })
    };
    let normal = chat(State(Arc::clone(&fixture.state)), None, request()).await;
    let stream = chat_stream(State(Arc::clone(&fixture.state)), None, request()).await;
    let count = fixture.request_count.load(Ordering::Relaxed);
    let permits = fixture.state.concurrency.available_permits();
    let inflight = fixture.worker.counter();
    fixture.server.stop().await.expect("upstream stops");
    assert_eq!(normal.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(stream.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(count, 0);
    assert_eq!(inflight, 0);
    assert_eq!(permits, 2);
}

#[tokio::test]
async fn poisoned_registry_rejects_management_and_stops_health_task() {
    let fixture = chat_fixture(ChatFixtureConfig::default()).await;
    poison(&fixture.state.registry);
    let health = management::health(State(Arc::clone(&fixture.state)))
        .await
        .into_response();
    let added = management::add_worker(
        State(Arc::clone(&fixture.state)),
        Json(WorkerConfig {
            id: "new".to_string(),
            address: "http://127.0.0.1:8001".to_string(),
        }),
    )
    .await;
    let removed = management::remove_worker(
        State(Arc::clone(&fixture.state)),
        Path("worker-a".to_string()),
    )
    .await;
    let task =
        management::start_health_checks(Arc::clone(&fixture.state), Duration::from_millis(10));
    let mut task = crate::gateway::tests::support::ManagedTask::new(task);
    // 等待任务正常退出；超时时仍由 ManagedTask 的 Drop 请求取消。
    let stopped = tokio::time::timeout(Duration::from_secs(1), task.join()).await;
    let count = fixture.request_count.load(Ordering::Relaxed);
    fixture.server.stop().await.expect("upstream stops");
    assert_eq!(health.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(added.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(removed, StatusCode::SERVICE_UNAVAILABLE);
    stopped
        .expect("health task stops promptly")
        .expect("health task does not panic");
    assert_eq!(count, 0);
}
