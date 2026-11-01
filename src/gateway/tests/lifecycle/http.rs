//! 真实 HTTP 链路：先确认首段到达，再制造中途失败或客户端断开。

use super::*;
use std::sync::atomic::AtomicUsize;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

const FIRST_EVENT: &[u8] = b"data: first\n\n";

pub(super) struct ObservedUpstream {
    address: String,
    task: ManagedTask<()>,
    pub(super) request_count: Arc<AtomicUsize>,
    pub(super) fail: Option<oneshot::Sender<()>>,
    peer_closed: oneshot::Receiver<io::Result<()>>,
}

pub(super) struct HttpStreamFixture {
    pub(super) upstream: ObservedUpstream,
    gateway: TestServer,
    pub(super) worker: Arc<Worker>,
    pub(super) state: Arc<GatewayState>,
}

impl HttpStreamFixture {
    pub(super) async fn stop(self) {
        self.gateway.stop().await.expect("gateway should stop");
        let error = self
            .upstream
            .task
            .cancel_and_join()
            .await
            .expect_err("upstream should still be listening");
        assert!(error.is_cancelled(), "upstream task must not panic");
    }
}

/// 消费完整请求，避免把尚未读取的 JSON 正文误判成后续连接活动。
async fn read_request(socket: &mut TcpStream) {
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("request line");
    assert_eq!(line, "POST /chat/stream HTTP/1.1\r\n");
    let mut content_length = None;
    loop {
        line.clear();
        let count = reader.read_line(&mut line).await.expect("request header");
        assert!(count > 0, "request headers must be complete");
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = Some(value.trim().parse::<usize>().expect("content length"));
            }
        }
    }
    let length = content_length.expect("test request should have a content length");
    assert!(length <= 1024, "test request should stay small");
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await.expect("request body");
    serde_json::from_slice::<ChatRequest>(&body).expect("valid chat request");
}

async fn observed_upstream(send_first_chunk: bool) -> ObservedUpstream {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("upstream should bind");
    let address = format!(
        "http://{}",
        listener.local_addr().expect("upstream address")
    );
    let request_count = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&request_count);
    let (fail, mut fail_rx) = oneshot::channel();
    let (closed_tx, peer_closed) = oneshot::channel();

    let task = ManagedTask::new(tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("upstream connection");
        read_request(&mut socket).await;
        count.fetch_add(1, Ordering::Relaxed);
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\
                  Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .await
            .expect("upstream response headers");
        if send_first_chunk {
            let length = format!("{:x}\r\n", FIRST_EVENT.len());
            socket
                .write_all(length.as_bytes())
                .await
                .expect("chunk size");
            socket.write_all(FIRST_EVENT).await.expect("first event");
            socket.write_all(b"\r\n").await.expect("chunk ending");
        }

        let mut buffer = [0; 1];
        tokio::select! {
            signal = &mut fail_rx => {
                signal.expect("failure must be triggered explicitly");
                // 不发送 chunked 正文的结束块，制造正文截断。
                drop(socket);
                drop(closed_tx);
            }
            read = socket.read(&mut buffer) => {
                let closure = match read {
                    Ok(0) => Ok(()),
                    Err(error) if matches!(
                        error.kind(),
                        io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
                    ) => Ok(()),
                    Err(error) => Err(error),
                    Ok(_) => Err(io::Error::other("unexpected data after the request")),
                };
                let _ = closed_tx.send(closure);
                drop(socket);
            }
        }

        // 保持监听。若网关重试，真实记录请求并返回可重试的 503。
        loop {
            let (mut socket, _) = listener.accept().await.expect("later connection");
            read_request(&mut socket).await;
            count.fetch_add(1, Ordering::Relaxed);
            let _ = socket
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\
                      Connection: close\r\n\r\n",
                )
                .await;
        }
    }));

    ObservedUpstream {
        address,
        task,
        request_count,
        fail: Some(fail),
        peer_closed,
    }
}

pub(super) async fn http_stream_fixture(send_first_chunk: bool) -> HttpStreamFixture {
    let upstream = observed_upstream(send_first_chunk).await;
    let worker = Arc::new(Worker::new(
        "worker-a".to_string(),
        upstream.address.clone(),
    ));
    let state = gateway_state_with_workers(&[Arc::clone(&worker)]);
    let gateway = start_test_server(
        Router::new()
            .route("/chat/stream", post(chat_stream))
            .with_state(Arc::clone(&state)),
    )
    .await;
    HttpStreamFixture {
        upstream,
        gateway,
        worker,
        state,
    }
}

#[tokio::test]
async fn partial_body_failure_does_not_retry_or_replay_over_http() {
    let mut fixture = http_stream_fixture(true).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .build()
        .expect("HTTP client");
    let mut response = tokio::time::timeout(
        Duration::from_secs(1),
        client
            .post(format!("{}/chat/stream", fixture.gateway.address()))
            .json(&ChatRequest {
                message: "partial failure".to_string(),
            })
            .send(),
    )
    .await
    .expect("response headers should arrive promptly")
    .expect("HTTP request should succeed");
    let status = response.status();
    let first = tokio::time::timeout(Duration::from_secs(1), async {
        let mut event = Vec::new();
        // HTTP 正文块不一定与 SSE 事件一一对应。
        while event.len() < FIRST_EVENT.len() {
            let chunk = response
                .chunk()
                .await
                .expect("first event should be readable")
                .expect("first event should exist");
            event.extend_from_slice(&chunk);
        }
        event
    })
    .await
    .expect("first event should arrive promptly");
    let before_failure = (
        fixture.worker.counter(),
        fixture.state.concurrency.available_permits(),
    );

    // 客户端已收到首段，才允许上游断开，避免首段与错误合并后尚未写出。
    fixture
        .upstream
        .fail
        .take()
        .expect("failure signal")
        .send(())
        .expect("upstream should be running");
    let mut extra = Vec::new();
    let remainder = tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(chunk) = response.chunk().await? {
            extra.extend_from_slice(&chunk);
        }
        Ok::<_, reqwest::Error>(())
    })
    .await;
    let released = wait_until(Duration::from_secs(1), || {
        fixture.worker.counter() == 0 && fixture.state.concurrency.available_permits() == 2
    })
    .await;
    let requests = fixture.upstream.request_count.load(Ordering::Relaxed);
    let after_failure = (
        fixture.worker.counter(),
        fixture.state.concurrency.available_permits(),
    );
    drop(response);
    drop(client);
    fixture.stop().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(first, FIRST_EVENT);
    assert_eq!(before_failure, (1, 1));
    let error = remainder
        .expect("truncated body should fail promptly")
        .expect_err("truncation must not look like normal EOF");
    // Reqwest 的 chunk() 把正文 frame 错误归为 Decode。
    assert!(
        error.is_decode() && !error.is_timeout(),
        "client should observe a truncated body, not a timeout: {error}"
    );
    assert!(extra.is_empty(), "failed stream must not replay any events");
    released.expect("failed body should release resources");
    assert_eq!(after_failure, (0, 2));
    assert_eq!(requests, 1, "partial response must not be retried");
}

/// 原始 TCP 客户端，销毁它就关闭真实连接，不受 HTTP 客户端连接池影响。
async fn connect_stream_client(gateway: &TestServer) -> (BufReader<TcpStream>, String) {
    let address = gateway
        .address()
        .strip_prefix("http://")
        .expect("HTTP address");
    let mut socket = TcpStream::connect(address)
        .await
        .expect("gateway connection");
    let body = br#"{"message":"disconnect"}"#;
    let headers = format!(
        "POST /chat/stream HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len(),
    );
    socket
        .write_all(headers.as_bytes())
        .await
        .expect("request headers");
    socket.write_all(body).await.expect("request body");
    let mut reader = BufReader::new(socket);
    let mut status = String::new();
    reader
        .read_line(&mut status)
        .await
        .expect("response status");
    loop {
        let mut line = String::new();
        let count = reader.read_line(&mut line).await.expect("response header");
        assert!(count > 0, "response headers must be complete");
        if line == "\r\n" {
            break;
        }
    }
    (reader, status)
}

async fn read_first_event(reader: &mut BufReader<TcpStream>) -> Vec<u8> {
    let mut event = Vec::new();
    while !event.ends_with(b"\n\n") {
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("chunk size");
        let length = usize::from_str_radix(line.trim(), 16).expect("hex chunk size");
        assert!(
            length > 0 && event.len() + length <= 1024,
            "event should be small"
        );
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes).await.expect("body chunk");
        let mut ending = [0; 2];
        reader.read_exact(&mut ending).await.expect("chunk ending");
        assert_eq!(&ending, b"\r\n");
        event.extend_from_slice(&bytes);
    }
    event
}

#[rstest]
#[case::before_first_event(false)]
#[case::after_first_event(true)]
#[tokio::test]
async fn client_disconnect_closes_upstream_and_releases_resources(#[case] read_first: bool) {
    let mut fixture = http_stream_fixture(read_first).await;
    let deadline = TokioInstant::now() + Duration::from_secs(1);
    let (mut client, status) =
        tokio::time::timeout_at(deadline, connect_stream_client(&fixture.gateway))
            .await
            .expect("response headers should arrive promptly");
    if read_first {
        let event = tokio::time::timeout_at(deadline, read_first_event(&mut client))
            .await
            .expect("first event should arrive promptly");
        assert_eq!(event, FIRST_EVENT);
    }
    let before_disconnect = (
        fixture.worker.counter(),
        fixture.state.concurrency.available_permits(),
    );
    drop(client);

    // 从请求开始总共只等 1 秒，不能靠 2 秒空闲超时替代断开清理。
    let cleanup = tokio::time::timeout_at(deadline, async {
        let closure = (&mut fixture.upstream.peer_closed).await;
        let released = wait_until(Duration::from_secs(1), || {
            fixture.worker.counter() == 0 && fixture.state.concurrency.available_permits() == 2
        })
        .await;
        (closure, released)
    })
    .await;
    let after_disconnect = (
        fixture.worker.counter(),
        fixture.state.concurrency.available_permits(),
    );
    let requests = fixture.upstream.request_count.load(Ordering::Relaxed);
    // 观测完成后才停止服务器，避免把测试主动清理误认为客户端断开生效。
    fixture.stop().await;

    assert_eq!(status, "HTTP/1.1 200 OK\r\n");
    assert_eq!(before_disconnect, (1, 1));
    let (closure, released) = cleanup.expect("disconnect should clean up before the idle timeout");
    closure
        .expect("upstream should report its peer closing")
        .expect("upstream should observe EOF or connection reset");
    released.expect("disconnect should release resources");
    assert_eq!(after_disconnect, (0, 2));
    assert_eq!(requests, 1);
}

#[rstest]
#[case::write_timeout(Duration::from_millis(500), true)]
#[case::control(Duration::from_secs(30), false)]
#[tokio::test]
async fn slow_client_write_timeout_releases_stream_resources(
    #[case] write_timeout: Duration,
    #[case] should_timeout: bool,
) {
    use std::sync::atomic::AtomicBool;
    use crate::write_timeout::WriteTimeoutListener;

    struct UpstreamBodyGuard(Arc<AtomicBool>);
    impl Drop for UpstreamBodyGuard {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    let closed = Arc::new(AtomicBool::new(false));
    let produced = Arc::new(AtomicUsize::new(0));
    let upstream_closed = Arc::clone(&closed);
    let upstream_produced = Arc::clone(&produced);
    let upstream = start_test_server(Router::new().route(
        "/chat/stream",
        post(move || {
            let guard = UpstreamBodyGuard(Arc::clone(&upstream_closed));
            let produced = Arc::clone(&upstream_produced);
            async move {
                let bytes = axum::body::Bytes::from(vec![b'x'; 64 * 1024]);
                let chunks = stream::unfold((guard, produced, bytes), |state| async move {
                    state.1.fetch_add(1, Ordering::Relaxed);
                    Some((Ok::<_, io::Error>(state.2.clone()), state))
                });
                ([(CONTENT_TYPE, "text/event-stream")], Body::from_stream(chunks))
            }
        }),
    )).await;
    let (worker, state) = stream_worker_state(&upstream);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new().route("/chat/stream", post(chat_stream)).with_state(Arc::clone(&state));
    let gateway = ManagedTask::new(tokio::spawn(async move {
        axum::serve(WriteTimeoutListener::new(listener, write_timeout), app).await.unwrap();
    }));
    let mut socket = TcpStream::connect(address).await.unwrap();
    let body = br#"{"message":"slow"}"#;
    let request = format!("POST /chat/stream HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n", body.len());
    socket.write_all(request.as_bytes()).await.unwrap();
    socket.write_all(body).await.unwrap();
    let mut client = BufReader::new(socket);
    let mut status = String::new();
    tokio::time::timeout(Duration::from_secs(2), client.read_line(&mut status)).await.unwrap().unwrap();
    loop {
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(2), client.read_line(&mut line)).await.unwrap().unwrap();
        if line == "\r\n" { break; }
        assert!(!line.is_empty(), "response headers must be complete");
    }
    // 保持连接但不读取正文，直到发送缓冲区被填满。
    let observed = wait_until(Duration::from_secs(4), || {
        worker.counter() == 0 && state.concurrency.available_permits() == 2
            && closed.load(Ordering::Relaxed)
    }).await;
    let chunks_produced = produced.load(Ordering::Relaxed);
    let inflight = worker.counter();
    let permits = state.concurrency.available_permits();
    drop(client);
    let gateway_result = gateway.cancel_and_join().await;
    let upstream_result = upstream.stop().await;

    assert_eq!(status, "HTTP/1.1 200 OK\r\n");
    assert!(chunks_produced > 1, "upstream must send enough data to exercise backpressure");
    if should_timeout {
        assert!(observed.is_ok(), "stalled write should close upstream and release resources");
        assert_eq!((inflight, permits), (0, 2));
    } else {
        assert!(observed.is_err(), "without a short write deadline the stream should remain active");
        assert_eq!((inflight, permits), (1, 1));
    }
    assert!(gateway_result.unwrap_err().is_cancelled());
    upstream_result.expect("upstream server should stop");
}
