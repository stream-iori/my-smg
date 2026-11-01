use super::is_retryable_status;
use axum::http::StatusCode;
use std::{future::Future, time::Duration};

type HttpResponse = (StatusCode, String);

pub(super) async fn retry_with_backoff<F, Fut>(
    mut operation: F,
) -> Result<(StatusCode, String), HttpResponse>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<HttpResponse, HttpResponse>>,
{
    const MAX_ATTEMPTS: usize = 3;
    let mut attempt = 1;
    loop {
        // 调用闭包创建新的 Future，再等待这一次操作完成。
        // Err 通过 ? 直接返回，不参与状态码重试。
        let (status, body) = operation().await?;

        if !is_retryable_status(status) || attempt >= MAX_ATTEMPTS {
            return Ok((status, body));
        }

        tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;

        attempt += 1;
    }
}
