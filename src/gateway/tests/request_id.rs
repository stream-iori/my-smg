//! 验证请求 ID 穿过中间件，并与响应头一致。
use crate::gateway::tests::support::start_test_server;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

use crate::{RequestId, request_id_middleware};
use axum::{Extension, middleware};

#[tokio::test]
async fn request_extension_matches_response_header_for_concurrent_requests() {
    let app = Router::new()
        .route(
            "/id",
            get(|Extension(RequestId(id)): Extension<RequestId>| async move { id.to_string() }),
        )
        .layer(middleware::from_fn(request_id_middleware));
    let server = start_test_server(app).await;
    let client = reqwest::Client::new();
    let url = format!("{}/id", server.address());
    let responses = futures_util::future::join_all((0..20).map(|_| async {
        let response = client.get(&url).send().await.expect("request succeeds");
        let status = response.status();
        let id = response
            .headers()
            .get("x-request-id")
            .expect("ID header")
            .to_str()
            .unwrap()
            .to_owned();
        let body = response.text().await.expect("response body");
        (status, id, body)
    }))
    .await;
    server.stop().await.expect("server should stop");

    let mut ids = std::collections::HashSet::new();
    for (status, id, body) in responses {
        assert_eq!(status, StatusCode::OK);
        assert_eq!(id, body, "handler extension and response header must match");
        assert!(ids.insert(id), "concurrent requests must have distinct IDs");
    }
}
