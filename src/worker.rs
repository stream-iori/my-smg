//! 共享节点与请求计数凭证。

use crate::circuit_gate::CircuitGate;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// 最近一次健康探测结果，与熔断状态独立。
#[derive(Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

impl Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => {
                write!(f, "Healthy")
            }
            Self::Unhealthy => {
                write!(f, "Unhealthy")
            }
        }
    }
}

/// 共享节点；健康状态、在途计数和熔断入口由所有请求共同观察。
#[derive(Debug)]
pub struct Worker {
    id: String,
    address: String,
    healthy: AtomicBool,
    counter: AtomicUsize,
    circuit_gate: Arc<CircuitGate>,
}

/// 借用节点的计数凭证；销毁时减一，必须覆盖完整逻辑请求。
#[must_use = "keep the guard until the request finishes"]
pub struct InFlightGuard<'worker> {
    worker: &'worker Worker,
}

impl<'worker> Drop for InFlightGuard<'worker> {
    fn drop(&mut self) {
        self.worker.decrement_count();
    }
}

/// 持有节点所有权的计数凭证，可随响应正文跨越处理器返回边界。
#[must_use = "keep the guard until the response body finishes"]
pub struct OwnedInFlightGuard {
    worker: Arc<Worker>,
}

impl Drop for OwnedInFlightGuard {
    fn drop(&mut self) {
        self.worker.decrement_count();
    }
}

impl Worker {
    /// 新节点初始健康；使用固定的三次失败阈值和五秒冷却期。
    pub fn new(id: String, address: String) -> Self {
        let circuit_gate = CircuitGate::try_new(3, Duration::from_secs(5))
            .expect("fixed circuit breaker configuration must be valid");

        Self {
            id,
            address,
            healthy: AtomicBool::new(true),
            counter: AtomicUsize::new(0),
            circuit_gate: Arc::new(circuit_gate),
        }
    }

    /// 返回共享入口句柄，供申请拥有独立生命周期的熔断凭证。
    pub fn circuit_gate(&self) -> &Arc<CircuitGate> {
        &self.circuit_gate
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    /// 当前逻辑请求数，包含重试和正文读取，排队阶段不计入。
    pub fn counter(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }

    /// 开始一次逻辑请求；必须保留返回的凭证直到请求结束。
    pub fn begin_request(&self) -> InFlightGuard<'_> {
        self.increment_count();
        InFlightGuard { worker: self }
    }

    /// 开始一次由凭证持有节点的请求，适用于流式正文。
    pub fn begin_request_owned(self: &Arc<Self>) -> OwnedInFlightGuard {
        self.increment_count();
        OwnedInFlightGuard {
            worker: Arc::clone(self),
        }
    }

    fn increment_count(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_count(&self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn status(&self) -> HealthStatus {
        if self.healthy.load(Ordering::Relaxed) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    pub fn set_status(&self, status: HealthStatus) {
        let healthy = match status {
            HealthStatus::Healthy => true,
            HealthStatus::Unhealthy => false,
        };

        self.healthy.store(healthy, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Instant;

    use super::{HealthStatus, Worker};
    use crate::circuit_breaker::CircuitState;

    #[test]
    fn shares_circuit_gate_between_arc_owners() {
        let worker = Arc::new(Worker::new(
            "worker-a".to_string(),
            "http://127.0.0.1:3001".to_string(),
        ));
        let another_owner = Arc::clone(&worker);

        // 两个持有者访问的是同一个 Gate 对象。
        assert!(Arc::ptr_eq(
            worker.circuit_gate(),
            another_owner.circuit_gate(),
        ));

        let now = Instant::now();

        for _ in 0..3 {
            let permit = worker
                .circuit_gate()
                .acquire(now)
                .expect("gate should be usable")
                .expect("request should be allowed");

            permit.failure(now).expect("should record failure");
        }

        // 通过另一持有者，可以观察到同一份熔断状态。
        assert_eq!(another_owner.circuit_gate().state(), Ok(CircuitState::Open),);
        assert!(
            another_owner
                .circuit_gate()
                .acquire(now)
                .expect("gate should be usable")
                .is_none()
        );
    }

    #[test]
    fn circuit_failure_does_not_affect_other_workers() {
        let worker_a = Worker::new("worker-a".to_string(), "http://127.0.0.1:3001".to_string());
        let worker_b = Worker::new("worker-b".to_string(), "http://127.0.0.1:3002".to_string());

        assert!(!Arc::ptr_eq(
            worker_a.circuit_gate(),
            worker_b.circuit_gate(),
        ));

        let now = Instant::now();

        for _ in 0..3 {
            let permit = worker_a
                .circuit_gate()
                .acquire(now)
                .expect("gate should be usable")
                .expect("request should be allowed");

            permit.failure(now).expect("should record failure");
        }

        assert_eq!(worker_a.circuit_gate().state(), Ok(CircuitState::Open),);
        assert_eq!(worker_b.circuit_gate().state(), Ok(CircuitState::Closed),);

        let permit_b = worker_b
            .circuit_gate()
            .acquire(now)
            .expect("gate should be usable")
            .expect("other worker should still allow requests");

        permit_b.success(now).expect("should record success");
    }

    #[test]
    fn counts_requests_per_worker() {
        let worker_a = Worker::new("worker-a".to_string(), "127.0.0.1:3212".to_string());
        let worker_b = Worker::new("worker-b".to_string(), "127.0.0.1:3213".to_string());

        assert_eq!(worker_a.counter(), 0);
        assert_eq!(worker_b.counter(), 0);

        {
            let _guard = worker_a.begin_request();
            assert_eq!(worker_a.counter(), 1);
            assert_eq!(worker_b.counter(), 0);
        }

        assert_eq!(worker_a.counter(), 0);
    }

    #[test]
    fn new_worker_is_healthy() {
        let worker = Worker::new("worker-a".to_string(), "127.0.0.1:3212".to_string());

        assert_eq!(worker.id(), "worker-a");
        assert_eq!(worker.status(), HealthStatus::Healthy);
    }

    #[test]
    fn worker_status_can_change() {
        let worker = Worker::new("worker-a".to_string(), "127.0.0.1:3212".to_string());

        worker.set_status(HealthStatus::Unhealthy);
        assert_eq!(worker.status(), HealthStatus::Unhealthy);

        worker.set_status(HealthStatus::Healthy);
        assert_eq!(worker.status(), HealthStatus::Healthy);
    }

    #[test]
    fn counts_owned_requests_until_guard_is_dropped() {
        let worker = Arc::new(Worker::new(
            "worker-a".to_string(),
            "http://127.0.0.1:3001".to_string(),
        ));

        assert_eq!(worker.counter(), 0);

        {
            let _guard = worker.begin_request_owned();

            assert_eq!(worker.counter(), 1);

            // 多一个共享句柄，不代表多一个在途请求。
            let another_owner = Arc::clone(&worker);
            assert_eq!(another_owner.counter(), 1);
        }

        assert_eq!(worker.counter(), 0);
    }

    #[test]
    fn owned_guard_outlives_its_creation_scope() {
        let guard = {
            let worker = Arc::new(Worker::new(
                "worker-a".to_string(),
                "http://127.0.0.1:3001".to_string(),
            ));

            worker.begin_request_owned()
        }; // 局部 worker 句柄销毁，但 guard 仍持有 Worker。

        assert_eq!(guard.worker.counter(), 1);

        // 保留一个观察句柄，用来检查 guard 销毁后的计数。
        let observer = Arc::clone(&guard.worker);

        let moved_guard = guard;
        assert_eq!(observer.counter(), 1);

        drop(moved_guard);

        assert_eq!(observer.counter(), 0);
    }

    #[test]
    fn shares_health_status_between_arc_owners() {
        let worker = std::sync::Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3212".to_string(),
        ));
        let another_owner = std::sync::Arc::clone(&worker);

        // 两个 Arc 共享同一个 Worker，因此从任意一方修改健康状态，另一方都能看到。
        assert_eq!(worker.status(), HealthStatus::Healthy);

        worker.set_status(HealthStatus::Unhealthy);
        assert_eq!(another_owner.status(), HealthStatus::Unhealthy);

        another_owner.set_status(HealthStatus::Healthy);
        assert_eq!(worker.status(), HealthStatus::Healthy);
    }
}
