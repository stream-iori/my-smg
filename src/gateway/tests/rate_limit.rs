use crate::gateway::chat::chat;
use crate::gateway::gateway_state::{ChatRequest, GatewayState};
use crate::gateway::tests::support::{ChatFixture, ChatFixtureConfig, chat_fixture};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::error::Elapsed;

/// 只封装调用与测试超时，断言留在各测试中。
async fn call_chat(state: &Arc<GatewayState>) -> Result<(StatusCode, String), Elapsed> {
    tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(Arc::clone(state)),
            None,
            Json(ChatRequest {
                message: "rate-limit-test".to_string(),
            }),
        ),
    )
    .await
}

#[tokio::test]
async fn concurrent_requests_share_one_bucket() {
    let config = ChatFixtureConfig {
        token_capacity: 2,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let (first, second, third) =
        tokio::join!(call_chat(&state), call_chat(&state), call_chat(&state),);

    // 先清理服务器，再检查结果。
    let server_result = server.stop().await;

    let responses = [
        first.expect("first request should finish"),
        second.expect("second request should finish"),
        third.expect("third request should finish"),
    ];

    let accepted = responses
        .iter()
        .filter(|response| response.0 == StatusCode::OK)
        .count();

    let rejected = responses
        .iter()
        .filter(|response| response.0 == StatusCode::TOO_MANY_REQUESTS)
        .count();

    assert_eq!(accepted, 2);
    assert_eq!(rejected, 1);

    for (status, body) in responses {
        if status == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(body, "rate limit exceeded");
        }
    }

    // 被限流的请求没有联系上游。
    assert_eq!(request_count.load(Ordering::Relaxed), 2);
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn retries_consume_only_one_token() {
    let config = ChatFixtureConfig {
        failures_before_success: 2,
        token_capacity: 1,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let first = call_chat(&state).await;
    let count_after_first = request_count.load(Ordering::Relaxed);

    let second = call_chat(&state).await;
    let count_after_second = request_count.load(Ordering::Relaxed);

    let server_result = server.stop().await;

    let (first_status, first_body) = first.expect("first request should finish");

    // 只有一枚令牌，但同一次逻辑请求仍能重试至成功。
    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(first_body, "successful response");
    assert_eq!(count_after_first, 3);

    let (second_status, second_body) = second.expect("second request should finish");

    assert_eq!(second_status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(second_body, "rate limit exceeded");

    // 第二次逻辑请求被拒绝，上游次数不再增加。
    assert_eq!(count_after_second, count_after_first);
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop without panicking");
}
