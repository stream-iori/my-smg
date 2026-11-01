//! 节点管理：注册、重复 ID、非法地址、移除及移除后的可见行为。

use super::*;

#[tokio::test]
async fn registers_valid_worker() {
    let state = empty_gateway_state();

    let (status, body) = add_worker(
        State(Arc::clone(&state)),
        Json(WorkerConfig {
            id: "worker-a".to_string(),
            address: "http://127.0.0.1:3001".to_string(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, "worker registered");

    let worker = {
        let registry = state
            .registry
            .lock()
            .expect("registry lock should not be poisoned");

        registry
            .get("worker-a")
            .expect("registered worker should exist")
    };

    assert_eq!(worker.address(), "http://127.0.0.1:3001");
}

#[tokio::test]
async fn duplicate_registration_preserves_original_worker() {
    let original = Arc::new(Worker::new(
        "worker-a".to_string(),
        "http://127.0.0.1:3001".to_string(),
    ));

    let state = gateway_state_with_workers(&[Arc::clone(&original)]);

    let (status, body) = add_worker(
        State(Arc::clone(&state)),
        Json(WorkerConfig {
            id: "worker-a".to_string(),
            address: "http://127.0.0.1:3002".to_string(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, "worker id already exists");

    let found = {
        let registry = state
            .registry
            .lock()
            .expect("registry lock should not be poisoned");

        assert_eq!(registry.snapshot().len(), 1);

        registry
            .get("worker-a")
            .expect("original worker should remain")
    };

    assert!(Arc::ptr_eq(&original, &found));
    assert_eq!(found.address(), "http://127.0.0.1:3001");
}

#[tokio::test]
async fn rejects_invalid_address_without_registering_worker() {
    let state = empty_gateway_state();

    let (status, body) = add_worker(
        State(Arc::clone(&state)),
        Json(WorkerConfig {
            id: "worker-a".to_string(),
            address: "not-a-url".to_string(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.starts_with("invalid worker:"));

    let registry = state
        .registry
        .lock()
        .expect("registry lock should not be poisoned");

    assert!(registry.snapshot().is_empty());
}

#[tokio::test]
async fn removes_worker_once_and_returns_404_afterwards() {
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        "http://127.0.0.1:3001".to_string(),
    ));

    let state = gateway_state_with_workers(&[worker]);

    let first = remove_worker(State(Arc::clone(&state)), AxumPath("worker-a".to_string())).await;

    assert_eq!(first, StatusCode::NO_CONTENT);

    let output = health(State(Arc::clone(&state))).await;
    assert_eq!(output, "ready: 0 workers\n");

    let second = remove_worker(State(Arc::clone(&state)), AxumPath("worker-a".to_string())).await;

    assert_eq!(second, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn handlers_reflect_worker_removal() {
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        "http://127.0.0.1:3001".to_string(),
    ));

    let state = gateway_state_with_workers(&[worker]);

    let before = health(State(Arc::clone(&state))).await;

    assert_eq!(
        before,
        "ready: 1 workers\nworker-a: inflight=0 health=Healthy\n"
    );

    {
        let mut registry = state
            .registry
            .lock()
            .expect("registry lock should not be poisoned");

        assert!(registry.remove("worker-a"));
    }

    let after = health(State(Arc::clone(&state))).await;

    assert_eq!(after, "ready: 0 workers\n");

    let (status, body) = chat(
        State(Arc::clone(&state)),
        None,
        Json(ChatRequest {
            message: "hello".to_string(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "no healthy worker");
}
