//! 流式读取期限：用真实 HTTP 分段响应验证计时和资源释放。
use super::{request_stream, stream_fixture, stream_worker_state};
use crate::gateway::gateway_state::ChatRequest;
use crate::gateway::tests::support::{TestServer, start_test_server};
use axum::body::Body;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::routing::post;
use axum::{Json, Router};
use futures_util::StreamExt;
use futures_util::stream;
use rstest::rstest;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant as TokioInstant;

/// 每个延迟对应一块正文；最后可保持连接打开，模拟上游停止输出。
async fn chunk_server(delays: Vec<Duration>, stall_after_chunks: bool) -> TestServer {
    let app = Router::new().route(
        "/chat/stream",
        post(move |Json(_request): Json<ChatRequest>| {
            let delays = delays.clone();
            async move {
                let chunks = stream::unfold(0, move |index| {
                    let delay = delays.get(index).copied();
                    async move {
                        match delay {
                            Some(delay) => {
                                tokio::time::sleep(delay).await;
                                Some((
                                    Ok::<_, std::convert::Infallible>("data: tick\n\n"),
                                    index + 1,
                                ))
                            }
                            None if stall_after_chunks => std::future::pending().await,
                            None => None,
                        }
                    }
                });
                (
                    [(CONTENT_TYPE, "text/event-stream")],
                    Body::from_stream(chunks),
                )
            }
        }),
    );
    start_test_server(app).await
}

fn assert_timeout_error(error: &axum::Error, message: &str) {
    let mut source: &(dyn std::error::Error + 'static) = error;
    loop {
        if let Some(error) = source.downcast_ref::<io::Error>() {
            assert_eq!(error.kind(), io::ErrorKind::TimedOut);
            assert_eq!(error.to_string(), message);
            return;
        }
        source = source
            .source()
            .expect("body error should preserve the underlying io::Error");
    }
}

#[rstest]
#[case::before_first_chunk(false)]
#[case::after_first_chunk(true)]
#[tokio::test]
async fn read_idle_timeout_releases_resources(#[case] send_first_chunk: bool) {
    let delays = if send_first_chunk {
        vec![Duration::ZERO]
    } else {
        vec![]
    };
    let server = chunk_server(delays, true).await;
    let (worker, state) = stream_worker_state(&server);
    let response = request_stream(Arc::clone(&state)).await;
    let status = response.status();
    let mut body = response.into_body().into_data_stream();

    if send_first_chunk {
        let first = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .expect("first chunk should arrive promptly")
            .expect("first chunk should exist")
            .expect("first chunk should be readable");
        assert_eq!(first.as_ref(), b"data: tick\n\n");
    }
    let before_read = (worker.counter(), state.concurrency.available_permits());
    let started = TokioInstant::now();
    let next = tokio::time::timeout(Duration::from_secs(4), body.next()).await;
    let elapsed = started.elapsed();
    // 不先 drop(body)：必须由超时分支自身释放资源。
    let after_timeout = (worker.counter(), state.concurrency.available_permits());
    let end = tokio::time::timeout(Duration::from_secs(1), body.next()).await;
    drop(body);
    let stopped = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(before_read, (1, 1));
    let error = next
        .expect("idle timeout should finish the read")
        .expect("stream should yield an error")
        .expect_err("stalled upstream must not finish successfully");
    assert_timeout_error(&error, "upstream body read idle timed out");
    assert!(elapsed >= Duration::from_secs(2));
    assert_eq!(after_timeout, (0, 2));
    assert!(end.expect("failed stream should terminate").is_none());
    stopped.expect("test server should stop");
}

#[tokio::test]
async fn read_idle_budget_resets_after_each_chunk() {
    // 每块间隔短于 2 秒，但整个正文超过 2 秒，应正常结束。
    let server = chunk_server(vec![Duration::from_millis(900); 3], false).await;
    let (worker, state) = stream_worker_state(&server);
    let response = request_stream(Arc::clone(&state)).await;
    let status = response.status();
    let mut body = response.into_body().into_data_stream();
    let collected = tokio::time::timeout(Duration::from_secs(5), async {
        let mut bytes = Vec::new();
        while let Some(chunk) = body.next().await {
            bytes.extend_from_slice(&chunk?);
        }
        Ok::<_, axum::Error>(bytes)
    })
    .await;
    let after_end = (worker.counter(), state.concurrency.available_permits());
    drop(body);
    let stopped = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    let bytes = collected
        .expect("body should finish in time")
        .expect("each chunk should reset the idle budget");
    assert_eq!(bytes, b"data: tick\n\n".repeat(3));
    assert_eq!(after_end, (0, 2));
    stopped.expect("test server should stop");
}

#[tokio::test]
async fn total_deadline_does_not_reset_when_chunks_keep_arriving() {
    // 持续输出超过 10 秒，任意两块之间都不会触发 2 秒空闲超时。
    let server = chunk_server(vec![Duration::from_millis(500); 30], false).await;
    let (worker, state) = stream_worker_state(&server);
    let started = TokioInstant::now();
    let response = request_stream(Arc::clone(&state)).await;
    let status = response.status();
    let mut body = response.into_body().into_data_stream();
    let mut received = Vec::new();
    let result = tokio::time::timeout(Duration::from_secs(12), async {
        while let Some(chunk) = body.next().await {
            received.extend_from_slice(&chunk?);
        }
        Ok::<_, axum::Error>(())
    })
    .await;
    let elapsed = started.elapsed();
    let after_timeout = (worker.counter(), state.concurrency.available_permits());
    drop(body);
    let stopped = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    let error = result
        .expect("fixed total deadline should stop the stream")
        .expect_err("regular chunks must not extend the total deadline");
    assert_timeout_error(&error, "upstream total deadline exceeded");
    // 确认并非 1 秒或 2 秒就误停，且已有多块正文成功转发。
    assert!(elapsed >= Duration::from_secs(10));
    assert!(received.len() >= b"data: tick\n\n".len() * 3);
    assert_eq!(after_timeout, (0, 2));
    stopped.expect("test server should stop");
}

#[tokio::test]
async fn expired_total_deadline_is_checked_before_reading_buffered_body() {
    let (server, worker, state) = stream_fixture().await;
    let response = request_stream(Arc::clone(&state)).await;
    let status = response.status();
    let mut body = response.into_body().into_data_stream();

    // 正文已可读，但暂时不拉取；期限应包含两次拉取之间的时间。
    tokio::time::sleep(Duration::from_millis(10_100)).await;
    let before_poll = (worker.counter(), state.concurrency.available_permits());
    let next = tokio::time::timeout(Duration::from_secs(1), body.next()).await;
    let after_poll = (worker.counter(), state.concurrency.available_permits());
    drop(body);
    let stopped = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    // 当前方案依靠拉取推进：未 poll 时，不会主动销毁正文。
    assert_eq!(before_poll, (1, 1));
    let error = next
        .expect("expired deadline should be detected promptly")
        .expect("stream should report the expired deadline")
        .expect_err("buffered data must not bypass an expired deadline");
    assert_timeout_error(&error, "upstream total deadline exceeded");
    assert_eq!(after_poll, (0, 2));
    stopped.expect("test server should stop");
}
