//! 网关共享状态与请求准入；锁仅覆盖同步访问，不跨网络等待。
use axum::http::StatusCode;
use my_smg::policy::Policy;
use my_smg::token_bucket::TokenBucket;
use my_smg::worker::Worker;
use my_smg::worker_registry::WorkerRegistry;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

#[derive(Deserialize, Serialize)]
pub(crate) struct ChatRequest {
    pub(super) message: String,
}

pub(crate) struct GatewayState {
    pub(crate) client: reqwest::Client,
    pub(crate) registry: Mutex<WorkerRegistry>,
    pub(crate) policy: Mutex<Box<dyn Policy + Send>>,
    pub(crate) token_bucket: Mutex<TokenBucket>,
    pub(crate) concurrency: Arc<Semaphore>,
    pub(crate) waiting: Arc<Semaphore>,
    pub(crate) queue_timeout: Duration,
}

fn unavailable(message: &str) -> (StatusCode, String) {
    (StatusCode::SERVICE_UNAVAILABLE, message.to_string())
}

impl GatewayState {
    // 一次逻辑请求扣一次额度；排队凭证不延长到执行阶段。
    pub(super) async fn acquire_request_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit, (StatusCode, String)> {
        let allowed = {
            let mut bucket = self
                .token_bucket
                .lock()
                .map_err(|_| unavailable("rate limiter unavailable"))?;
            bucket.try_acquire(Instant::now())
        };
        if !allowed {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "rate limit exceeded".to_string(),
            ));
        }
        match Arc::clone(&self.concurrency).try_acquire_owned() {
            Ok(permit) => Ok(permit),
            Err(TryAcquireError::Closed) => Err(unavailable("gateway concurrency unavailable")),
            Err(TryAcquireError::NoPermits) => {
                let _waiting_permit = self.waiting.try_acquire().map_err(|error| match error {
                    TryAcquireError::Closed => unavailable("gateway queue unavailable"),
                    TryAcquireError::NoPermits => unavailable("gateway queue full"),
                })?;
                tokio::time::timeout(
                    self.queue_timeout,
                    Arc::clone(&self.concurrency).acquire_owned(),
                )
                .await
                .map_err(|_| unavailable("gateway queue wait timed out"))?
                .map_err(|_| unavailable("gateway concurrency unavailable"))
            }
        }
    }

    // 使用同一份快照解释策略返回的索引，返回共享节点句柄。
    pub(super) fn select_worker(&self) -> Result<Arc<Worker>, (StatusCode, String)> {
        let snapshot = self
            .registry
            .lock()
            .map_err(|_| unavailable("worker registry unavailable"))?
            .snapshot();
        let mut policy = self
            .policy
            .lock()
            .map_err(|_| unavailable("worker policy unavailable"))?;
        let index = policy
            .select(&snapshot)
            .ok_or_else(|| unavailable("no healthy worker"))?;
        Ok(Arc::clone(&snapshot[index]))
    }
}
