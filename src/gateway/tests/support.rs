//! 测试工具：准备状态、启动假服务、管理任务生命周期和等待条件。
//! 工具不替测试判断业务结果，断言保留在各测试用例中。
use crate::gateway::gateway_state::{ChatRequest, GatewayState};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use my_smg::policy::RoundRobin;
use my_smg::token_bucket::TokenBucket;
use my_smg::worker::Worker;
use my_smg::worker_registry::WorkerRegistry;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use std::sync::atomic::AtomicUsize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

/// 正常路径取消并等待任务；提前退出时 Drop 只负责请求取消。
pub(super) struct ManagedTask<T> {
    handle: Option<JoinHandle<T>>,
}

impl<T> ManagedTask<T> {
    pub(super) fn new(handle: JoinHandle<T>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    pub(super) async fn join(&mut self) -> Result<T, JoinError> {
        let result = self
            .handle
            .as_mut()
            .expect("managed task owns its handle")
            .await;
        self.handle.take();
        result
    }

    //`mut self` 不是 `&mut self`：方法取得对象所有权，调用后原变量不能再使用
    pub(super) async fn cancel_and_join(mut self) -> Result<T, JoinError> {
        // take() 把句柄移出并留下 None，避免之后 Drop 再次处理同一个句柄。
        let handle = self
            .handle
            .take()
            .expect("managed task should own its handle");
        handle.abort();
        // abort 只是请求取消；await 返回后，才能确认任务和它的 guard 已结束。
        handle.await
    }
}

impl<T> Drop for ManagedTask<T> {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            // Drop 不能异步等待。这是提前退出的兜底，不代替正常路径的清理。
            handle.abort();
        }
    }
}

/// 地址与服务器任务一起持有，避免只保存地址而忘记清理后台任务。
pub(super) struct TestServer {
    address: String,
    task: ManagedTask<()>,
}

impl TestServer {
    pub(super) fn address(&self) -> &str {
        &self.address
    }

    pub(super) async fn stop(self) -> Result<(), JoinError> {
        match self.task.cancel_and_join().await {
            Ok(()) => Ok(()),
            // 服务器允许已经完成；取消属于正常清理，但 panic 不能被吞掉。
            Err(error) if error.is_cancelled() => Ok(()),
            Err(error) => Err(error),
        }
    }
}

/// 每次调用 chat_fixture 都创建独立的节点、计数器、令牌桶和信号量。
pub(super) struct ChatFixture {
    pub(super) server: TestServer,
    pub(super) worker: Arc<Worker>,
    pub(super) state: Arc<GatewayState>,
    pub(super) request_count: Arc<AtomicUsize>,
}

pub(super) struct ChatFixtureConfig {
    pub(super) failures_before_success: usize,
    pub(super) failure_status: StatusCode,
    pub(super) response_delay: Duration,
    pub(super) queue_capacity: usize,
    pub(super) token_capacity: usize,
}

impl Default for ChatFixtureConfig {
    fn default() -> Self {
        Self {
            failures_before_success: 0,
            failure_status: StatusCode::SERVICE_UNAVAILABLE,
            response_delay: Duration::ZERO,
            queue_capacity: 0,
            token_capacity: 100,
        }
    }
}

pub(super) async fn chat_fixture(config: ChatFixtureConfig) -> ChatFixture {
    let (server, request_count) = start_retry_server(
        config.failures_before_success,
        config.failure_status,
        config.response_delay,
    )
    .await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));
    let bucket = TokenBucket::try_new(
        config.token_capacity,
        Duration::from_secs(60),
        Instant::now(),
    )
    .expect("valid test bucket configuration");
    let state = gateway_state_with_bucket_and_queue(
        &[Arc::clone(&worker)],
        bucket,
        config.queue_capacity,
        Duration::from_millis(300),
    );

    ChatFixture {
        server,
        worker,
        state,
        request_count,
    }
}

pub(super) struct HealthFixture {
    pub(super) server: TestServer,
    pub(super) worker: Arc<Worker>,
    pub(super) state: Arc<GatewayState>,
}

pub(super) async fn health_fixture(status: StatusCode) -> HealthFixture {
    let server = start_health_server(status).await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        server.address().to_string(),
    ));
    let state = gateway_state_with_workers(&[Arc::clone(&worker)]);

    HealthFixture {
        server,
        worker,
        state,
    }
}

pub(super) fn idle_worker() -> Arc<Worker> {
    Arc::new(Worker::new(
        "worker-a".to_string(),
        "127.0.0.1:3212".to_string(),
    ))
}

/// 普通测试使用充足额度，避免限流干扰其他行为的验收。
pub(super) fn gateway_state_with_workers(workers: &[Arc<Worker>]) -> Arc<GatewayState> {
    let bucket = TokenBucket::try_new(100, Duration::from_secs(60), Instant::now())
        .expect("valid test bucket configuration");

    gateway_state_with_bucket(workers, bucket)
}

pub(super) fn gateway_state_with_bucket_and_queue(
    workers: &[Arc<Worker>],
    bucket: TokenBucket,
    queue_capacity: usize,
    queue_timeout: Duration,
) -> Arc<GatewayState> {
    let mut registry = WorkerRegistry::new();

    for worker in workers {
        assert!(
            registry.insert(Arc::clone(worker)),
            "fixture worker IDs should be unique",
        );
    }

    Arc::new(GatewayState {
        // 本地假节点必须直连，不能让系统代理把连接失败转成 HTTP 502。
        client: reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("test HTTP client should build"),
        registry: Mutex::new(registry),
        policy: Mutex::new(Box::new(RoundRobin::new())),
        token_bucket: Mutex::new(bucket),
        concurrency: Arc::new(Semaphore::new(2)),
        waiting: Arc::new(Semaphore::new(queue_capacity)),
        queue_timeout,
    })
}

/// 限流测试可以传入自己的桶。
pub(super) fn gateway_state_with_bucket(
    workers: &[Arc<Worker>],
    bucket: TokenBucket,
) -> Arc<GatewayState> {
    gateway_state_with_bucket_and_queue(workers, bucket, 0, Duration::from_millis(300))
}

pub(super) async fn start_retry_server(
    failures_before_success: usize,
    failure_status: StatusCode,
    response_delay: Duration,
) -> (TestServer, Arc<AtomicUsize>) {
    let request_count = Arc::new(AtomicUsize::new(0));
    let handler_count = Arc::clone(&request_count);

    let app = Router::new().route(
        "/chat",
        post(move |Json(_request): Json<ChatRequest>| {
            // 先读取请求体，避免未消费的请求体影响后续连接复用。
            // 每次请求取得自己的 Arc，再移入本次 Future。
            let request_count = Arc::clone(&handler_count);

            async move {
                // fetch_add 返回增加前的值：第一条请求得到 0。
                let previous_count = request_count.fetch_add(1, Ordering::Relaxed);

                tokio::time::sleep(response_delay).await;

                if previous_count < failures_before_success {
                    (failure_status, "simulated failure")
                } else {
                    (StatusCode::OK, "successful response")
                }
            }
        }),
    );

    let server = start_test_server(app).await;

    (server, request_count)
}

pub(super) fn empty_gateway_state() -> Arc<GatewayState> {
    gateway_state_with_workers(&[])
}

/// 使用随机端口，保证多个测试可以并行运行而不争抢固定端口。
pub(super) async fn start_test_server(app: Router) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");

    let address = listener
        .local_addr()
        .expect("test server should have an address");

    let task = ManagedTask::new(tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("test server should run");
    }));

    TestServer {
        address: format!("http://{address}"),
        task,
    }
}

pub(super) async fn start_health_server(status: StatusCode) -> TestServer {
    let app = Router::new().route("/health", get(move || async move { status }));
    start_test_server(app).await
}

pub(super) async fn start_slow_chat_server(delay: Duration) -> TestServer {
    let app = Router::new().route(
        "/chat",
        post(move || async move {
            tokio::time::sleep(delay).await;
            "slow response"
        }),
    );
    start_test_server(app).await
}

/// 分别控制响应头和正文的时间，才能专门覆盖 text().await 的超时分支。
pub(super) async fn start_slow_body_server(delay: Duration) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("test server should have an address");
    let task = ManagedTask::new(tokio::spawn(async move {
        let (mut socket, _) = listener
            .accept()
            .await
            .expect("test server should accept a connection");
        let mut received = Vec::new();
        let mut buffer = [0u8; 1024];

        loop {
            let count = socket
                .read(&mut buffer)
                .await
                .expect("test server should read the request");
            assert!(
                count > 0,
                "connection closed before request headers arrived"
            );
            // TCP 可分多次送达；只累积本次读到的字节，直到请求头结束。
            received.extend_from_slice(&buffer[..count]);
            if received.windows(4).any(|part| part == b"\r\n\r\n") {
                break;
            }
        }

        // 声明正文还有 5 字节：收到 200 响应头不代表完整响应已收到。
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n")
            .await
            .expect("test server should send response headers");
        tokio::time::sleep(delay).await;
        // 客户端超时后可能已经断开，因此允许这次正文写入失败。
        let _ = socket.write_all(b"hello").await;
    }));

    TestServer {
        address: format!("http://{address}"),
        task,
    }
}

/// 等待可观察状态，不靠固定睡眠猜测任务是否启动；超时结果由测试自行断言。
pub(super) async fn wait_until(
    limit: Duration,
    mut condition: impl FnMut() -> bool,
) -> Result<(), Elapsed> {
    tokio::time::timeout(limit, async {
        while !condition() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
}
