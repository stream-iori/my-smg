//! 健康检查：探测失败标记不健康，后续成功探测恢复健康。
use crate::gateway::management::{check_worker_health, start_health_checks};
use crate::gateway::tests::support::{HealthFixture, ManagedTask, health_fixture, wait_until};
use axum::http::StatusCode;
use my_smg::worker::HealthStatus;
use std::sync::Arc;
use std::time::Duration;

use rstest::rstest;

#[rstest]
#[case::recovery(StatusCode::OK, HealthStatus::Unhealthy, HealthStatus::Healthy)]
#[case::failure(
    StatusCode::SERVICE_UNAVAILABLE,
    HealthStatus::Healthy,
    HealthStatus::Unhealthy
)]
#[tokio::test]
async fn health_probe_updates_worker_status(
    #[case] status: StatusCode,
    #[case] initial: HealthStatus,
    #[case] expected: HealthStatus,
) {
    let HealthFixture {
        server,
        worker,
        state,
    } = health_fixture(status).await;
    let default_status = worker.status();
    worker.set_status(initial);

    check_worker_health(&state.client, &worker).await;
    let actual = worker.status();

    server
        .stop()
        .await
        .expect("test server should stop without panicking");
    assert_eq!(default_status, HealthStatus::Healthy);
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn background_health_checks_recover_worker() {
    let HealthFixture {
        server,
        worker,
        state,
    } = health_fixture(StatusCode::OK).await;
    worker.set_status(HealthStatus::Unhealthy);

    let health_task = ManagedTask::new(start_health_checks(
        Arc::clone(&state),
        Duration::from_millis(10),
    ));

    let recovered = wait_until(Duration::from_secs(3), || {
        worker.status() == HealthStatus::Healthy
    })
    .await;

    let health_result = health_task.cancel_and_join().await;
    let server_result = server.stop().await;

    assert!(
        recovered.is_ok(),
        "background checks should recover the worker"
    );

    let health_error = health_result.expect_err("health task should be cancelled");
    assert!(health_error.is_cancelled());

    server_result.expect("test server should stop without panicking");

    assert_eq!(worker.status(), HealthStatus::Healthy);
}
