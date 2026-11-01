//! 转发行为：可重试状态、退避、单次/总超时，以及取消后停止请求。
//! 外围 timeout 是测试保险，chat 内部的 timeout 才是被测行为。

use super::*;
use rstest::rstest;

#[tokio::test]
async fn waits_retry_before_upstream() {
    let config = ChatFixtureConfig {
        failures_before_success: 2,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let started = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(state),
            None,
            Json(ChatRequest {
                message: "retry-request".to_string(),
            }),
        ),
    )
    .await;
    let elapsed = started.elapsed();

    // 先清理，再断言，避免断言失败跳过清理。
    let server_result = server.stop().await;
    let (status, body) = result.expect("chat should finish within the test deadline");

    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body, "successful response");
    assert_eq!(request_count.load(Ordering::Relaxed), 3);
    assert!(
        elapsed >= Duration::from_millis(300),
        "expected at least 300ms of backoff, actual: {elapsed:?}",
    );
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop without panicking");
}

#[rstest]
#[case::recovers(2, StatusCode::SERVICE_UNAVAILABLE, StatusCode::OK, 3)]
#[case::attempt_limit(3, StatusCode::SERVICE_UNAVAILABLE, StatusCode::SERVICE_UNAVAILABLE, 3)]
#[case::bad_request(3, StatusCode::BAD_REQUEST, StatusCode::BAD_REQUEST, 1)]
#[tokio::test]
async fn retries_only_retryable_responses_within_attempt_limit(
    #[case] failures: usize,
    #[case] failure_status: StatusCode,
    #[case] expected_status: StatusCode,
    #[case] expected_count: usize,
) {
    let config = ChatFixtureConfig {
        failures_before_success: failures,
        failure_status,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let result = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(state),
            None,
            Json(ChatRequest {
                message: "retry-check".to_string(),
            }),
        ),
    )
    .await;

    // 先清理，再断言；测试失败也不会跳过正常清理步骤。
    let server_result = server.stop().await;
    let (status, body) = result.expect("chat should finish within the test deadline");

    assert_eq!(
        status,
        expected_status,
        "response body: {body}; worker received {} requests",
        request_count.load(Ordering::Relaxed),
    );

    let expected_body = if expected_status == StatusCode::OK {
        "successful response"
    } else {
        "simulated failure"
    };

    assert_eq!(body, expected_body);
    assert_eq!(request_count.load(Ordering::Relaxed), expected_count);
    assert_eq!(worker.counter(), 0);
    assert_eq!(worker.status(), HealthStatus::Healthy);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn returns_503_when_no_worker_is_healthy() {
    let worker = Arc::new(Worker::new(
        "worker-1".to_string(),
        "http://127.0.0.1:3001".to_string(),
    ));
    worker.set_status(HealthStatus::Unhealthy);

    let state = gateway_state_with_workers(&[worker]);

    let (status, body) = chat(
        State(state),
        None,
        Json(ChatRequest {
            message: "hello".to_string(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "no healthy worker");
}

#[tokio::test]
async fn returns_504_and_resets_count_when_upstream_is_slow() {
    let server = start_slow_chat_server(Duration::from_millis(1500)).await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));
    let state = gateway_state_with_workers(&[Arc::clone(&worker)]);

    // 3 秒是测试兜底；生产 chat 内的 1 秒才是请求超时。
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(Arc::clone(&state)),
            None,
            Json(ChatRequest {
                message: "timeout-check".to_string(),
            }),
        ),
    )
    .await;

    let server_result = server.stop().await;
    let (status, body) = result.expect("chat should finish within the test deadline");

    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(body, "upstream request timed out");
    assert_eq!(worker.counter(), 0);
    assert_eq!(worker.status(), HealthStatus::Healthy);
    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn returns_504_when_retry_total_deadline_expires() {
    let config = ChatFixtureConfig {
        failures_before_success: 2,
        response_delay: Duration::from_millis(750),
        ..Default::default()
    };
    // 每次响应不到 1 秒，但三次请求加退避会超过总期限。
    let ChatFixture {
        server,
        worker,
        state,
        ..
    } = chat_fixture(config).await;

    // 4 秒是测试保险；被测的是 chat 内部的 2 秒总期限。
    let result = tokio::time::timeout(
        Duration::from_secs(4),
        chat(
            State(state),
            None,
            Json(ChatRequest {
                message: "total-deadline-check".to_string(),
            }),
        ),
    )
    .await;

    // 先清理，再断言。
    let server_result = server.stop().await;

    let (status, body) = result.expect("chat should finish within the test deadline");

    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(body, "upstream total deadline exceeded");
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn returns_504_when_upstream_body_is_slow() {
    let server = start_slow_body_server(Duration::from_millis(1500)).await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));
    let state = gateway_state_with_workers(&[Arc::clone(&worker)]);

    let result = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(Arc::clone(&state)),
            None,
            Json(ChatRequest {
                message: "body-timeout-check".to_string(),
            }),
        ),
    )
    .await;

    let server_result = server.stop().await;
    let (status, body) = result.expect("chat should finish within the test deadline");

    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    // response 而非 request：证明超时发生在读取正文时。
    assert_eq!(body, "upstream response timed out");
    assert_eq!(worker.counter(), 0);
    assert_eq!(worker.status(), HealthStatus::Healthy);
    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn stops_retrying_when_chat_is_cancelled() {
    let config = ChatFixtureConfig {
        failures_before_success: 10,
        ..Default::default()
    };
    // 持续返回可重试的 503，保留请求计数供测试观察。
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;
    let request_state = Arc::clone(&state);

    let request_task = ManagedTask::new(tokio::spawn(async move {
        chat(
            State(request_state),
            None,
            Json(ChatRequest {
                message: "cancel-retry-check".to_string(),
            }),
        )
        .await
    }));

    // 等待假 Worker 真正收到请求，而不是只观察在途计数。
    let started = wait_until(Duration::from_secs(2), || {
        request_count.load(Ordering::Relaxed) >= 1
    })
    .await;

    // 等待任务结束后，才能确认 guard 已经释放。
    let request_result = request_task.cancel_and_join().await;
    let count_after_cancel = request_count.load(Ordering::Relaxed);

    // 保持服务器运行，观察是否还有后续请求。
    tokio::time::sleep(Duration::from_millis(400)).await;
    let count_after_observation = request_count.load(Ordering::Relaxed);

    //assert after clean
    let server_result = server.stop().await;

    started.expect("worker should receive the first request");
    let error = request_result.expect_err("chat task should be cancelled");
    assert!(error.is_cancelled());
    assert_eq!(
        count_after_observation, count_after_cancel,
        "request count should not increase after cancellation",
    );
    assert_eq!(worker.counter(), 0);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn resets_count_when_chat_is_cancelled() {
    let server = start_slow_chat_server(Duration::from_secs(60)).await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));
    let state = gateway_state_with_workers(&[Arc::clone(&worker)]);
    let request_state = Arc::clone(&state);

    let request_task = ManagedTask::new(tokio::spawn(async move {
        chat(
            State(request_state),
            None,
            Json(ChatRequest {
                message: "cancel-check".to_string(),
            }),
        )
        .await
    }));

    // 计数为 1 表示 guard 已创建，不表示假 Worker 已经收到请求。
    let started = wait_until(Duration::from_secs(2), || worker.counter() == 1).await;

    // 必须在断言前清理两个任务；取消 chat 的结果仍由本测试明确验证。
    let request_result = request_task.cancel_and_join().await;
    let server_result = server.stop().await;

    started.expect("chat should become in flight");
    let error = request_result.expect_err("chat task should be cancelled");
    assert!(error.is_cancelled());
    assert_eq!(worker.counter(), 0);
    assert_eq!(worker.status(), HealthStatus::Healthy);
    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn open_circuit_rejects_requests_without_contacting_worker() {
    use my_smg::circuit_breaker::CircuitState;

    let config = ChatFixtureConfig {
        failures_before_success: 10,
        ..Default::default()
    };

    // 持续失败，第一次 chat 的三次尝试会触发熔断。
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    let first = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(Arc::clone(&state)),
            None,
            Json(ChatRequest {
                message: "open-circuit".to_string(),
            }),
        ),
    )
    .await;

    // 继续使用同一个 Worker 和状态，不重新创建熔断器。
    let second = tokio::time::timeout(
        Duration::from_secs(3),
        chat(
            State(state),
            None,
            Json(ChatRequest {
                message: "rejected-by-circuit".to_string(),
            }),
        ),
    )
    .await;

    let server_result = server.stop().await;

    let (first_status, first_body) = first.expect("first chat should finish within the deadline");
    let (second_status, second_body) =
        second.expect("second chat should finish within the deadline");

    assert_eq!(first_status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(first_body, "simulated failure");

    assert_eq!(second_status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(second_body, "worker circuit rejected request");

    // 第二个 chat 被网关拒绝，没有产生任何新的上游请求。
    assert_eq!(request_count.load(Ordering::Relaxed), 3);
    assert_eq!(worker.circuit_gate().state(), Ok(CircuitState::Open),);
    assert_eq!(worker.counter(), 0);

    // 熔断与健康探测状态相互独立。
    assert_eq!(worker.status(), HealthStatus::Healthy);

    server_result.expect("test server should stop without panicking");
}

#[tokio::test]
async fn recovers_forwading_after_circuit_cooldown() {
    use my_smg::circuit_breaker::CircuitState;

    // 只封装请求构造与测试超时，不隐藏熔断流程，也不在清理前断言。
    async fn call_chat(
        state: &Arc<GatewayState>,
        message: &str,
    ) -> Result<(StatusCode, String), tokio::time::error::Elapsed> {
        tokio::time::timeout(
            Duration::from_secs(3),
            chat(
                State(Arc::clone(state)),
                None,
                Json(ChatRequest {
                    message: message.to_string(),
                }),
            ),
        )
        .await
    }

    // 准备：前三次上游请求失败，之后成功；全流程共享同一个 Worker。
    let config = ChatFixtureConfig {
        failures_before_success: 3,
        ..Default::default()
    };
    let ChatFixture {
        server,
        worker,
        state,
        request_count,
    } = chat_fixture(config).await;

    // 1. 三次失败触发熔断：计数和状态都在这一步结束时保存。
    let trip_result = call_chat(&state, "trip").await;
    let count_after_trip = request_count.load(Ordering::Relaxed);
    let state_after_trip = worker.circuit_gate().state();
    let inflight_after_trip = worker.counter();

    // 2. 冷却期间再次请求：应当本地拒绝，而不是继续访问上游。
    let rejected_result = call_chat(&state, "reject").await;
    let count_after_rejection = request_count.load(Ordering::Relaxed);
    let state_after_rejection = worker.circuit_gate().state();
    let inflight_after_rejection = worker.counter();

    // 3. 跨过 Worker 固定的5秒冷却，再用真实 HTTP 请求探测恢复。
    // 等待不属于某次 chat 的3秒测试超时。
    tokio::time::sleep(Duration::from_millis(5100)).await;
    let probe_result = call_chat(&state, "probe").await;
    let count_after_probe = request_count.load(Ordering::Relaxed);
    let state_after_probe = worker.circuit_gate().state();
    let inflight_after_probe = worker.counter();

    // 4. 恢复后再发一个请求，确认不仅探测成功，正常转发也已恢复。
    let normal_result = call_chat(&state, "normal").await;
    let count_after_recovery = request_count.load(Ordering::Relaxed);
    let state_after_recovery = worker.circuit_gate().state();
    let inflight_after_recovery = worker.counter();

    // 完成资源清理后再断言；后面的检查使用各阶段已保存的快照。
    let server_result = server.stop().await;

    // 第一阶段：上游失败响应原样返回，并打开熔断器。
    let (status, body) = trip_result.expect("trip request should finish within the deadline");
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "simulated failure");
    assert_eq!(count_after_trip, 3);
    assert_eq!(state_after_trip, Ok(CircuitState::Open));

    // 第二阶段：次数不变，证明熔断拒绝没有产生上游请求。
    let (status, body) =
        rejected_result.expect("rejected request should finish within the deadline");
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "worker circuit rejected request");
    assert_eq!(
        count_after_rejection, count_after_trip,
        "rejection should not contact the upstream",
    );
    assert_eq!(state_after_rejection, Ok(CircuitState::Open));

    // 第三阶段：只增加一次上游请求，探测成功后关闭熔断。
    let (status, body) = probe_result.expect("probe request should finish within the deadline");
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "successful response");
    assert_eq!(
        count_after_probe,
        count_after_rejection + 1,
        "recovery should require only one successful probe",
    );
    assert_eq!(state_after_probe, Ok(CircuitState::Closed));

    // 第四阶段：正常请求同样只发送一次。
    let (status, body) = normal_result.expect("normal request should finish within the deadline");
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "successful response");
    assert_eq!(
        count_after_recovery,
        count_after_probe + 1,
        "normal forwarding should send one upstream request",
    );
    assert_eq!(state_after_recovery, Ok(CircuitState::Closed));

    // 数据驱动只用于完全相同的不变量检查，不再控制执行顺序或等待。
    for (phase, inflight) in [
        ("trip", inflight_after_trip),
        ("reject", inflight_after_rejection),
        ("probe", inflight_after_probe),
        ("normal", inflight_after_recovery),
    ] {
        assert_eq!(inflight, 0, "in-flight count after {phase}");
    }

    server_result.expect("test server should stop without panicking");
}

#[rstest]
#[case::bad_gateway(StatusCode::BAD_GATEWAY, true)]
#[case::unavailable(StatusCode::SERVICE_UNAVAILABLE, true)]
#[case::gateway_timeout(StatusCode::GATEWAY_TIMEOUT, true)]
#[case::ok(StatusCode::OK, false)]
#[case::bad_request(StatusCode::BAD_REQUEST, false)]
#[case::unauthorized(StatusCode::UNAUTHORIZED, false)]
#[case::forbidden(StatusCode::FORBIDDEN, false)]
#[case::not_found(StatusCode::NOT_FOUND, false)]
#[case::internal_error(StatusCode::INTERNAL_SERVER_ERROR, false)]
#[case::rate_limited(StatusCode::TOO_MANY_REQUESTS, false)]
fn classifies_retryable_upstream_statuses(#[case] status: StatusCode, #[case] expected: bool) {
    assert_eq!(is_retryable_status(status), expected, "status: {status}");
}
