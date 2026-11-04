use crate::gateway::chat::chat;
use crate::gateway::gateway_state::{ChatRequest, GatewayState};
use crate::gateway::tests::support::{
    ChatFixture, ChatFixtureConfig, ManagedTask, chat_fixture, wait_until,
};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

fn spawn_chat(
    state: &Arc<GatewayState>,
    message: &'static str,
) -> ManagedTask<(StatusCode, String)> {
    let request_state = Arc::clone(state);

    ManagedTask::new(tokio::spawn(async move {
        chat(
            State(request_state),
            None,
            Json(ChatRequest {
                message: message.to_string(),
            }),
        )
        .await
    }))
}

async fn call_chat(state: &Arc<GatewayState>, message: &str) -> (StatusCode, String) {
    chat(
        State(Arc::clone(state)),
        None,
        Json(ChatRequest {
            message: message.to_string(),
        }),
    )
    .await
}

#[tokio::test]
async fn rejects_when_full_and_releases_permits_after_cancellation() {
    let config = ChatFixtureConfig {
        response_delay: Duration::from_secs(60),
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let task_a = spawn_chat(&state, "request-a");
    let task_b = spawn_chat(&state, "request-b");

    // 等上游真正收到两个请求，在网关的 1 秒超时之前检查。
    let started = wait_until(Duration::from_millis(500), || {
        request_count.load(Ordering::Relaxed) == 2
    })
    .await;

    let permits_before_c = state.concurrency.available_permits();
    let inflight_before_c = worker.counter();

    // 满额拒绝应立即完成；外层超时防止错误实现让测试一直等待。
    let response_c = tokio::time::timeout(
        Duration::from_millis(100),
        chat(
            State(Arc::clone(&state)),
            None,
            Json(ChatRequest {
                message: "request-c".to_string(),
            }),
        ),
    )
    .await;

    let count_after_c = request_count.load(Ordering::Relaxed);

    // 每取消一个任务，就应归还它持有的名额。
    let result_a = task_a.cancel_and_join().await;
    let permits_after_a = state.concurrency.available_permits();
    let inflight_after_a = worker.counter();

    let result_b = task_b.cancel_and_join().await;
    let permits_after_b = state.concurrency.available_permits();
    let inflight_after_b = worker.counter();

    // 清理完成后再断言，避免断言失败遗留后台任务。
    let server_result = server.stop().await;

    started.expect("both requests should reach upstream before timeout");
    assert_eq!(permits_before_c, 0);
    assert_eq!(inflight_before_c, 2);

    let (status_c, _) = response_c.expect("full gateway should reject immediately");

    assert_eq!(status_c, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(count_after_c, 2, "rejected request must not reach upstream");

    let error_a = result_a.expect_err("request A should be cancelled");
    assert!(error_a.is_cancelled());
    assert_eq!(permits_after_a, 1);
    assert_eq!(inflight_after_a, 1);

    let error_b = result_b.expect_err("request B should be cancelled");
    assert!(error_b.is_cancelled());
    assert_eq!(permits_after_b, 2);
    assert_eq!(inflight_after_b, 0);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn bounds_queue_and_forwards_after_capacity_is_released() {
    let config = ChatFixtureConfig {
        queue_capacity: 2,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    // 模拟两个执行名额已被占满。
    let occupied = state
        .concurrency
        .acquire_many(2)
        .await
        .expect("execution slots should be available");

    // join! 同时推进控制流程和两个排队请求。
    let (observed, response_c, response_d) = tokio::join!(
        async {
            let queued = wait_until(Duration::from_millis(200), || {
                state.waiting.available_permits() == 0
            })
            .await;

            // C、D 已占满等待名额，E 应立即被拒绝。
            let response_e =
                tokio::time::timeout(Duration::from_millis(100), call_chat(&state, "request-e"))
                    .await;

            let upstream_while_queued = request_count.load(Ordering::Relaxed);
            let inflight_while_queued = worker.counter();

            // 让 C、D 有机会取得执行名额。
            drop(occupied);

            (
                queued,
                response_e,
                upstream_while_queued,
                inflight_while_queued,
            )
        },
        call_chat(&state, "request-c"),
        call_chat(&state, "request-d"),
    );

    let count_after = request_count.load(Ordering::Relaxed);
    let server_result = server.stop().await;

    let (queued, response_e, upstream_while_queued, inflight_while_queued) = observed;

    queued.expect("both requests should enter the queue");

    assert_eq!(
        response_e.expect("full queue should reject immediately"),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "gateway queue full".to_string(),
        ),
    );

    // 排队中的请求尚未选择节点，也未联系上游。
    assert_eq!(upstream_while_queued, 0);
    assert_eq!(inflight_while_queued, 0);

    assert_eq!(response_c.0, StatusCode::OK);
    assert_eq!(response_d.0, StatusCode::OK);
    assert_eq!(count_after, 2);

    assert_eq!(state.waiting.available_permits(), 2);
    assert_eq!(state.concurrency.available_permits(), 2);
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn releases_waiting_slots_after_cancellation_and_timeout() {
    let config = ChatFixtureConfig {
        queue_capacity: 2,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let occupied = state
        .concurrency
        .acquire_many(2)
        .await
        .expect("execution slots should be available");

    // 先验证排队取消。
    let task = spawn_chat(&state, "cancel-queued-request");

    let queued = wait_until(Duration::from_millis(200), || {
        state.waiting.available_permits() == 1
    })
    .await;

    let cancel_result = task.cancel_and_join().await;
    let waiting_after_cancel = state.waiting.available_permits();

    // 执行名额仍被占用，下一条请求应等待到超时。
    let timed_out = tokio::time::timeout(
        Duration::from_secs(1),
        call_chat(&state, "timeout-queued-request"),
    )
    .await;

    let waiting_after_timeout = state.waiting.available_permits();
    let execution_after_timeout = state.concurrency.available_permits();
    let upstream_count = request_count.load(Ordering::Relaxed);
    let inflight = worker.counter();

    drop(occupied);
    let server_result = server.stop().await;

    queued.expect("request should enter the queue");

    let error = cancel_result.expect_err("queued task should be cancelled");
    assert!(error.is_cancelled());
    assert_eq!(waiting_after_cancel, 2);

    assert_eq!(
        timed_out.expect("queue timeout should finish within test deadline"),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "gateway queue wait timed out".to_string(),
        ),
    );

    assert_eq!(waiting_after_timeout, 2);
    assert_eq!(execution_after_timeout, 0);
    assert_eq!(upstream_count, 0);
    assert_eq!(inflight, 0);
    assert_eq!(state.concurrency.available_permits(), 2);

    server_result.expect("test server should stop");
}
