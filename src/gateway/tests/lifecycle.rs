//! 资源生命周期：多个任务共享同一 Worker，各自的 guard 独立释放。
//! 取消必须等任务结束；Drop 只请求取消时，要等待计数实际归零。

use super::*;
use futures_util::StreamExt;
use rstest::rstest;

mod http;
mod reliability;
mod timeouts;

fn stream_worker_state(server: &TestServer) -> (Arc<Worker>, Arc<GatewayState>) {
    stream_worker_state_with_queue(server, 0, Duration::from_millis(300))
}

fn stream_worker_state_with_queue(
    server: &TestServer,
    queue_capacity: usize,
    queue_timeout: Duration,
) -> (Arc<Worker>, Arc<GatewayState>) {
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));

    let bucket = TokenBucket::new(100, Duration::from_secs(60), Instant::now())
        .expect("valid test bucket configuration");

    let state = gateway_state_with_bucket_and_queue(
        &[Arc::clone(&worker)],
        bucket,
        queue_capacity,
        queue_timeout,
    );

    (worker, state)
}

async fn stream_fixture() -> (TestServer, Arc<Worker>, Arc<GatewayState>) {
    stream_fixture_with_queue(0, Duration::from_millis(300)).await
}

async fn stream_fixture_with_queue(
    queue_capacity: usize,
    queue_timeout: Duration,
) -> (TestServer, Arc<Worker>, Arc<GatewayState>) {
    let app = Router::new().route(
        "/chat/stream",
        post(|Json(_request): Json<ChatRequest>| async {
            ([(CONTENT_TYPE, "text/event-stream")], "data: hello\n\n")
        }),
    );

    let server = start_test_server(app).await;
    let (worker, state) = stream_worker_state_with_queue(&server, queue_capacity, queue_timeout);

    (server, worker, state)
}

async fn request_stream(state: Arc<GatewayState>) -> Response {
    tokio::time::timeout(
        Duration::from_secs(2),
        chat_stream(
            State(state),
            None,
            Json(ChatRequest {
                message: "stream-request".to_string(),
            }),
        ),
    )
    .await
    .expect("stream handler should return within the test deadline")
}

#[tokio::test]
async fn stream_count_lasts_until_body_reaches_end() {
    let (server, worker, state) = stream_fixture().await;

    let response = request_stream(Arc::clone(&state)).await;

    let status = response.status();
    let count_after_handler = worker.counter();
    let permits_after_handler = state.concurrency.available_permits();

    let mut body = response.into_body().into_data_stream();

    let collected = tokio::time::timeout(Duration::from_secs(2), async {
        let mut collected = Vec::new();

        while let Some(chunk) = body.next().await {
            let chunk = chunk.expect("body chunk should be readable");
            collected.extend_from_slice(&chunk);
        }

        collected
    })
    .await;

    // 在销毁 body 前记录，验证读取结束本身就能释放 guard 和执行名额。
    let count_after_end = worker.counter();
    let permits_after_end = state.concurrency.available_permits();

    drop(body);
    let server_result = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(count_after_handler, 1);
    assert_eq!(permits_after_handler, 1);

    let collected = collected.expect("body should finish in time");
    assert_eq!(collected.as_slice(), b"data: hello\n\n");

    assert_eq!(count_after_end, 0);
    assert_eq!(permits_after_end, 2);
    server_result.expect("test server should stop");
}

#[tokio::test]
async fn resets_count_when_task_is_cancelled() {
    let worker = idle_worker();
    let task_worker = Arc::clone(&worker);

    let task = ManagedTask::new(tokio::spawn(async move {
        let _guard = task_worker.begin_request();

        //pending::<()>() 产生一个永远不会完成的 Future，让任务停在等待状态
        std::future::pending::<()>().await;
    }));

    let started = wait_until(Duration::from_secs(2), || worker.counter() == 1).await;
    let result = task.cancel_and_join().await;
    assert!(started.is_ok(), "task should create its guard in time");

    let error = result.expect_err("task should be cancelled");
    assert!(error.is_cancelled());
    assert_eq!(worker.counter(), 0);
}

#[tokio::test]
async fn counts_two_concurrent_requests_for_same_worker() {
    let worker = idle_worker();
    let task_a_worker = Arc::clone(&worker);
    let task_a = ManagedTask::new(tokio::spawn(async move {
        let _guard = task_a_worker.begin_request();
        std::future::pending::<()>().await;
    }));

    let task_b_worker = Arc::clone(&worker);
    let task_b = ManagedTask::new(tokio::spawn(async move {
        let _guard = task_b_worker.begin_request();
        std::future::pending::<()>().await;
    }));

    let started = wait_until(Duration::from_secs(2), || worker.counter() == 2).await;

    let result_a = task_a.cancel_and_join().await;
    let remaining_after_a = worker.counter();

    let result_b = task_b.cancel_and_join().await;

    assert!(started.is_ok(), "both tasks should create their guards");

    let error_a = result_a.expect_err("task A should be cancelled");
    assert!(error_a.is_cancelled());
    assert_eq!(remaining_after_a, 1);

    let error_b = result_b.expect_err("task B should be cancelled");
    assert!(error_b.is_cancelled());
    assert_eq!(worker.counter(), 0);
}

#[tokio::test]
async fn resets_count_when_managed_task_is_dropped() {
    let worker = idle_worker();
    let task_worker = Arc::clone(&worker);
    let task = ManagedTask::new(tokio::spawn(async move {
        let _guard = task_worker.begin_request();
        std::future::pending::<()>().await;
    }));

    let started = wait_until(Duration::from_secs(2), || worker.counter() == 1).await;

    // 模拟测试提前退出：没有显式 join，Drop 仍应请求取消任务。
    drop(task);
    // Drop 不等待取消完成，因此不能马上断言计数为 0。
    let released = wait_until(Duration::from_secs(2), || worker.counter() == 0).await;

    started.expect("task should create its guard");
    released.expect("dropping the managed task should eventually release its guard");
    assert_eq!(worker.counter(), 0);
}

#[rstest]
#[case::unread(false)]
#[case::after_first_chunk(true)]
#[tokio::test]
async fn dropping_stream_body_releases_count(#[case] read_first_chunk: bool) {
    let (server, worker, state) = stream_fixture().await;

    let response = request_stream(Arc::clone(&state)).await;

    let status = response.status();
    let count_after_handler = worker.counter();
    let permits_after_handler = state.concurrency.available_permits();

    let mut body = response.into_body().into_data_stream();

    if read_first_chunk {
        let first = tokio::time::timeout(Duration::from_secs(2), body.next())
            .await
            .expect("first chunk should arrive in time")
            .expect("body should contain a chunk")
            .expect("first chunk should be readable");

        assert!(!first.is_empty());
    }

    // 没有继续拉取到 None，流仍持有 guard。
    let count_before_drop = worker.counter();
    let permits_before_drop = state.concurrency.available_permits();

    // 销毁正文流，连带销毁内部状态和 guard。
    drop(body);

    let count_after_drop = worker.counter();
    let permits_after_drop = state.concurrency.available_permits();

    // 正常路径先清理服务器，再检查结果。
    let server_result = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(count_after_handler, 1);
    assert_eq!(permits_after_handler, 1);
    assert_eq!(count_before_drop, 1);
    assert_eq!(permits_before_drop, 1);
    assert_eq!(count_after_drop, 0);
    assert_eq!(permits_after_drop, 2);

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_send_failure_releases_count() {
    let (server, worker, state) = stream_fixture().await;

    // 先停止服务器，使后续连接失败。
    server.stop().await.expect("test server should stop");

    assert_eq!(worker.counter(), 0);
    assert_eq!(state.concurrency.available_permits(), 2);

    let response = request_stream(Arc::clone(&state)).await;

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    // 发送失败时，handler 提前返回，局部 guard 和执行名额应已释放。
    assert_eq!(worker.counter(), 0);
    assert_eq!(state.concurrency.available_permits(), 2);
}

#[tokio::test]
async fn stream_bodies_hold_execution_permits() {
    let (server, worker, state) = stream_fixture().await;

    // 两个 handler 都返回，但正文还没有读取或销毁。
    let response_a = request_stream(Arc::clone(&state)).await;
    let response_b = request_stream(Arc::clone(&state)).await;

    let status_a = response_a.status();
    let status_b = response_b.status();

    let permits_when_full = state.concurrency.available_permits();
    let inflight_when_full = worker.counter();

    // 没有执行名额，等待容量也为 0。
    let response_c = request_stream(Arc::clone(&state)).await;

    let status_c = response_c.status();
    let inflight_after_rejection = worker.counter();

    // 销毁一个响应正文，释放一个执行名额。
    drop(response_a);

    let permits_after_a = state.concurrency.available_permits();
    let inflight_after_a = worker.counter();

    // 归还的名额应该可以被新请求使用。
    let response_d = request_stream(Arc::clone(&state)).await;

    let status_d = response_d.status();
    let permits_after_d = state.concurrency.available_permits();
    let inflight_after_d = worker.counter();

    drop(response_b);
    drop(response_d);
    drop(response_c);

    let permits_after_cleanup = state.concurrency.available_permits();
    let inflight_after_cleanup = worker.counter();

    let server_result = server.stop().await;

    assert_eq!(status_a, StatusCode::OK);
    assert_eq!(status_b, StatusCode::OK);
    assert_eq!(permits_when_full, 0);
    assert_eq!(inflight_when_full, 2);

    assert_eq!(status_c, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(inflight_after_rejection, 2);

    assert_eq!(permits_after_a, 1);
    assert_eq!(inflight_after_a, 1);

    assert_eq!(status_d, StatusCode::OK);
    assert_eq!(permits_after_d, 0);
    assert_eq!(inflight_after_d, 2);

    assert_eq!(permits_after_cleanup, 2);
    assert_eq!(inflight_after_cleanup, 0);

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_queue_releases_waiting_slot_after_admission() {
    let (server, worker, state) = stream_fixture_with_queue(1, Duration::from_secs(5)).await;

    // A、B 返回响应，但保留正文，占住两个执行名额。
    let response_a = request_stream(Arc::clone(&state)).await;
    let response_b = request_stream(Arc::clone(&state)).await;

    let status_a = response_a.status();
    let status_b = response_b.status();

    let execution_before_c = state.concurrency.available_permits();
    let waiting_before_c = state.waiting.available_permits();
    let inflight_before_c = worker.counter();

    // 同时推进观察流程和 C，避免只创建 Future 却没有执行。
    let (observed, response_c) = tokio::join!(
        async {
            let queued = wait_until(Duration::from_secs(1), || {
                state.waiting.available_permits() == 0
            })
            .await;

            // C 占用等待名额时，尚未增加节点在途数。
            let execution_while_queued = state.concurrency.available_permits();
            let waiting_while_queued = state.waiting.available_permits();
            let inflight_while_queued = worker.counter();

            // 归还 A 的执行名额，让 C 获准执行。
            drop(response_a);

            (
                queued,
                execution_while_queued,
                waiting_while_queued,
                inflight_while_queued,
            )
        },
        request_stream(Arc::clone(&state)),
    );

    let status_c = response_c.status();

    // B、C 的正文仍存活；等待名额已归还，执行名额继续占用。
    let execution_after_admission = state.concurrency.available_permits();
    let waiting_after_admission = state.waiting.available_permits();
    let inflight_after_admission = worker.counter();

    drop(response_b);
    drop(response_c);

    let execution_after_cleanup = state.concurrency.available_permits();
    let waiting_after_cleanup = state.waiting.available_permits();
    let inflight_after_cleanup = worker.counter();

    let server_result = server.stop().await;

    let (queued, execution_while_queued, waiting_while_queued, inflight_while_queued) = observed;

    assert_eq!(status_a, StatusCode::OK);
    assert_eq!(status_b, StatusCode::OK);

    assert_eq!(execution_before_c, 0);
    assert_eq!(waiting_before_c, 1);
    assert_eq!(inflight_before_c, 2);

    queued.expect("request C should enter the queue");

    assert_eq!(execution_while_queued, 0);
    assert_eq!(waiting_while_queued, 0);
    assert_eq!(inflight_while_queued, 2);

    assert_eq!(status_c, StatusCode::OK);
    assert_eq!(execution_after_admission, 0);
    assert_eq!(waiting_after_admission, 1);
    assert_eq!(inflight_after_admission, 2);

    assert_eq!(execution_after_cleanup, 2);
    assert_eq!(waiting_after_cleanup, 1);
    assert_eq!(inflight_after_cleanup, 0);

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_queue_cancellation_releases_waiting_slot() {
    let (server, worker, state) = stream_fixture_with_queue(1, Duration::from_secs(5)).await;

    // 保留 A、B 的正文，占满执行名额。
    let response_a = request_stream(Arc::clone(&state)).await;
    let response_b = request_stream(Arc::clone(&state)).await;

    let status_a = response_a.status();
    let status_b = response_b.status();

    // 创建并调度 C，让它进入排队。
    let request_c = request_stream(Arc::clone(&state));
    let task_c = ManagedTask::new(tokio::spawn(request_c));

    let queued = wait_until(Duration::from_secs(1), || {
        state.waiting.available_permits() == 0
    })
    .await;

    // 状态元组：(可用执行名额，可用等待名额，在途请求数)。
    let while_queued = (
        state.concurrency.available_permits(),
        state.waiting.available_permits(),
        worker.counter(),
    );

    // 等待任务退出后，再检查 Future 和等待凭证是否已清理。
    let cancel_result = task_c.cancel_and_join().await;

    let after_cancellation = (
        state.concurrency.available_permits(),
        state.waiting.available_permits(),
        worker.counter(),
    );

    drop(response_a);
    drop(response_b);

    let after_cleanup = (
        state.concurrency.available_permits(),
        state.waiting.available_permits(),
        worker.counter(),
    );

    let server_result = server.stop().await;

    assert_eq!(status_a, StatusCode::OK);
    assert_eq!(status_b, StatusCode::OK);

    queued.expect("request C should enter the queue");

    let error = cancel_result.expect_err("request C should be cancelled");
    assert!(error.is_cancelled());

    assert_eq!(while_queued, (0, 0, 2));
    assert_eq!(after_cancellation, (0, 1, 2));
    assert_eq!(after_cleanup, (2, 1, 0));

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_queue_timeout_releases_waiting_slot() {
    let (server, worker, state) = stream_fixture_with_queue(1, Duration::from_millis(100)).await;

    // A、B 的正文一直保留，让 C 无法取得执行名额。
    let response_a = request_stream(Arc::clone(&state)).await;
    let response_b = request_stream(Arc::clone(&state)).await;

    let status_a = response_a.status();
    let status_b = response_b.status();

    let response_c = request_stream(Arc::clone(&state)).await;
    let status_c = response_c.status();

    let after_timeout = (
        state.concurrency.available_permits(),
        state.waiting.available_permits(),
        worker.counter(),
    );

    // 有限错误正文，限制读取时间和字节数，便于核对超时分支。
    let body_result = tokio::time::timeout(
        Duration::from_secs(2),
        axum::body::to_bytes(response_c.into_body(), 1024),
    )
    .await;

    drop(response_a);
    drop(response_b);

    let after_cleanup = (
        state.concurrency.available_permits(),
        state.waiting.available_permits(),
        worker.counter(),
    );

    let server_result = server.stop().await;

    assert_eq!(status_a, StatusCode::OK);
    assert_eq!(status_b, StatusCode::OK);
    assert_eq!(status_c, StatusCode::SERVICE_UNAVAILABLE);

    let body = body_result
        .expect("error body should arrive in time")
        .expect("error body should be readable");

    assert_eq!(body.as_ref(), b"gateway queue wait timed out");

    assert_eq!(after_timeout, (0, 1, 2));
    assert_eq!(after_cleanup, (2, 1, 0));

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_header_timeout_releases_resources() {
    let app = Router::new().route(
        "/chat/stream",
        post(|Json(_request): Json<ChatRequest>| async {
            // 延迟返回响应头，超过网关的 1 秒期限。
            tokio::time::sleep(Duration::from_secs(5)).await;

            ([(CONTENT_TYPE, "text/event-stream")], "data: late\n\n")
        }),
    );

    let server = start_test_server(app).await;
    let (worker, state) = stream_worker_state(&server);

    let response = request_stream(Arc::clone(&state)).await;

    let status = response.status();
    let count_after_timeout = worker.counter();
    let permits_after_timeout = state.concurrency.available_permits();

    let body_result = tokio::time::timeout(
        Duration::from_secs(2),
        axum::body::to_bytes(response.into_body(), 1024),
    )
    .await;

    let server_result = server.stop().await;

    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(count_after_timeout, 0);
    assert_eq!(permits_after_timeout, 2);

    let body = body_result
        .expect("error body should arrive in time")
        .expect("error body should be readable");

    assert_eq!(body.as_ref(), b"upstream response headers timed out");

    server_result.expect("test server should stop");
}

#[tokio::test]
async fn stream_header_timeout_does_not_limit_body_reading() {
    let server = start_slow_body_server(Duration::from_millis(1500)).await;

    let (worker, state) = stream_worker_state(&server);

    let response = request_stream(Arc::clone(&state)).await;

    let status = response.status();
    let count_after_handler = worker.counter();
    let permits_after_handler = state.concurrency.available_permits();

    // 正文在 1.5 秒后到达，晚于响应头的 1 秒期限。
    let body_result = tokio::time::timeout(
        Duration::from_secs(3),
        axum::body::to_bytes(response.into_body(), 1024),
    )
    .await;

    let count_after_body = worker.counter();
    let permits_after_body = state.concurrency.available_permits();

    let server_result = server.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(count_after_handler, 1);
    assert_eq!(permits_after_handler, 1);

    let body = body_result
        .expect("body should arrive within the test deadline")
        .expect("header timeout must not interrupt body reading");

    assert_eq!(body.as_ref(), b"hello");
    assert_eq!(count_after_body, 0);
    assert_eq!(permits_after_body, 2);

    server_result.expect("test server should stop");
}

//已取得响应头，但读取正文失败，释放计数
#[tokio::test]
async fn stream_read_failure_releases_count() {
    // 服务器先发送响应头，再等待正文。
    let server = start_slow_body_server(Duration::from_secs(60)).await;

    let (worker, state) = stream_worker_state(&server);

    let response = request_stream(Arc::clone(&state)).await;

    let status = response.status();
    let count_after_handler = worker.counter();
    let permits_after_handler = state.concurrency.available_permits();

    let mut body = response.into_body().into_data_stream();

    // 响应声明还有 5 字节正文，但服务器尚未发送。
    // 此时停止服务器，制造正文不完整的读取错误。
    server.stop().await.expect("test server should stop");

    let next = tokio::time::timeout(Duration::from_secs(2), body.next()).await;

    // 必须在 drop(body) 之前记录。
    // 验证读取错误本身已经让流释放 guard 和执行名额。
    let count_after_error = worker.counter();
    let permits_after_error = state.concurrency.available_permits();

    drop(body);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(count_after_handler, 1);
    assert_eq!(permits_after_handler, 1);

    let item = next
        .expect("body read should finish in time")
        .expect("body should report a read error");

    assert!(item.is_err());
    assert_eq!(count_after_error, 0);
    assert_eq!(permits_after_error, 2);
}
