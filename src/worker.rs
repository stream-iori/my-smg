use crate::circuit_breaker::gate::CircuitGate;
use std::{
    fmt::Display,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

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

#[derive(Debug)]
pub struct Worker {
    id: String,
    address: String,
    healthy: AtomicBool,
    counter: AtomicUsize,
    circuit_gate: Arc<CircuitGate>,
}

pub struct InFlightGuard<'a> {
    worker: &'a Worker,
}

impl<'a> Drop for InFlightGuard<'a> {
    fn drop(&mut self) {
        self.worker.decrement_count();
    }
}

pub struct OwnedInFlightGuard {
    worker: Arc<Worker>,
}

impl Drop for OwnedInFlightGuard {
    fn drop(&mut self) {
        self.worker.decrement_count();
    }
}

impl Worker {
    pub fn new(id: String, address: String) -> Self {
        let circuit_gate = CircuitGate::new(3, Duration::from_secs(5))
            .expect("fixed circuit breaker configuration must be valid");

        Self {
            id,
            address,
            healthy: AtomicBool::new(true),
            counter: AtomicUsize::new(0),
            circuit_gate: Arc::new(circuit_gate),
        }
    }

    pub fn circuit_gate(&self) -> &Arc<CircuitGate> {
        &self.circuit_gate
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn counter(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }

    pub fn begin_request(&self) -> InFlightGuard<'_> {
        self.increment_count();
        InFlightGuard { worker: self }
    }

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
    use std::{sync::Arc, time::Instant};

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
