use super::{CircuitBreaker, CircuitState};
use std::{
    error::Error,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitGateError {
    Poisoned,
}

impl fmt::Display for CircuitGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned => write!(f, "circuit gate lock is poisoned"),
        }
    }
}

impl Error for CircuitGateError {}

/// 共享入口：负责锁保护、请求凭证和取消处理。
#[derive(Debug)]
pub struct CircuitGate {
    inner: Mutex<GateState>,
}

#[derive(Debug)]
struct GateState {
    breaker: CircuitBreaker,

    // 只使用对象身份，不需要保存业务数据。
    // 每次状态转换都换一个身份，旧凭证因此失效。
    generation: Arc<()>,
}

impl GateState {
    fn refresh_generation(&mut self, previous: CircuitState) {
        if self.breaker.state() != previous {
            self.generation = Arc::new(());
        }
    }
}

/// 必须保留到本次请求完成。
/// 不实现 Clone，避免同一个凭证被重复提交结果。
#[must_use = "keep the permit until the request finishes"]
pub struct CircuitPermit {
    gate: Arc<CircuitGate>,
    generation: Arc<()>,
    finished: bool,
}

impl CircuitGate {
    pub fn new(failure_threshold: usize, cooldown: Duration) -> Result<Self, &'static str> {
        let breaker = CircuitBreaker::new(failure_threshold, cooldown)?;

        Ok(Self {
            inner: Mutex::new(GateState {
                breaker,
                generation: Arc::new(()),
            }),
        })
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, GateState>, CircuitGateError> {
        self.inner.lock().map_err(|_| CircuitGateError::Poisoned)
    }

    pub fn state(&self) -> Result<CircuitState, CircuitGateError> {
        // 使用? 表示有错误就自己向上抛了
        let inner = self.lock_state()?;
        Ok(inner.breaker.state())
    }

    // 调用方需要通过 Arc 持有 gate。
    // 凭证也持有一个 Arc，确保 gate 比请求活得更久。
    pub fn acquire(
        self: &Arc<Self>,
        now: Instant,
    ) -> Result<Option<CircuitPermit>, CircuitGateError> {
        let mut inner = self.lock_state()?;
        let previous = inner.breaker.state();

        if !inner.breaker.allow_request(now) {
            return Ok(None);
        }

        // Open → HalfOpen 时换身份。
        // 旧的正常请求不能冒充本轮探测。
        inner.refresh_generation(previous);

        Ok(Some(CircuitPermit {
            gate: Arc::clone(self),
            generation: Arc::clone(&inner.generation),
            finished: false,
        }))
    }

    fn record_result(
        &self,
        generation: &Arc<()>,
        success: bool,
        now: Instant,
    ) -> Result<(), CircuitGateError> {
        let mut inner = self.lock_state()?;

        // 不属于当前轮次的结果，直接忽略。
        if !Arc::ptr_eq(generation, &inner.generation) {
            return Ok(());
        }

        let previous = inner.breaker.state();

        if success {
            inner.breaker.record_success();
        } else {
            inner.breaker.record_failure(now);
        }

        inner.refresh_generation(previous);
        Ok(())
    }

    fn cancel(&self, generation: &Arc<()>, now: Instant) -> Result<(), CircuitGateError> {
        let mut inner = self.lock_state()?;

        if !Arc::ptr_eq(generation, &inner.generation) {
            return Ok(());
        }

        // 普通请求取消不计入失败。
        // 探测取消则重新熔断，避免永久卡在 HalfOpen。
        if inner.breaker.state() == CircuitState::HalfOpen {
            let previous = inner.breaker.state();

            inner.breaker.record_failure(now);
            inner.refresh_generation(previous);
        }

        Ok(())
    }
}

impl CircuitPermit {
    // 消费凭证：调用后不能再对同一个凭证记录失败。
    pub fn success(mut self, now: Instant) -> Result<(), CircuitGateError> {
        // 表示调用方已经提交结果，不表示内部更新一定成功。
        // 更新失败时返回 Err，Drop 不再重复尝试。
        self.finished = true;
        self.gate.record_result(&self.generation, true, now)
    }

    pub fn failure(mut self, now: Instant) -> Result<(), CircuitGateError> {
        self.finished = true;
        self.gate.record_result(&self.generation, false, now)
    }
}

impl Drop for CircuitPermit {
    fn drop(&mut self) {
        if !self.finished {
            // Drop 无法返回 Result。
            // 毒锁时 cancel 在 ? 处结束，不读取或修改可疑状态。
            // 保留 poisoned 标记，后续 acquire 继续返回错误。
            let _ = self.gate.cancel(&self.generation, Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CircuitGate, CircuitGateError};
    use crate::circuit_breaker::CircuitState;
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    // 在另一个线程持锁时故意 panic，并等待线程退出，确保毒锁已生效。
    fn poison_gate(gate: &Arc<CircuitGate>) {
        assert!(!gate.inner.is_poisoned());
        let shared = Arc::clone(gate);

        let result = std::thread::spawn(move || {
            let _guard = shared
                .inner
                .lock()
                .expect("lock should initially be usable");
            panic!("intentional panic to poison the gate");
        })
        .join();

        assert!(result.is_err(), "poisoning thread should panic");
        assert!(gate.inner.is_poisoned(), "lock should now be poisoned");
    }

    #[test]
    fn old_result_cannot_complete_new_probe() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(1, Duration::from_secs(5)).expect("valid configuration"));

        // 两个请求在正常状态下先后获得放行。
        let old_request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("should allow request");
        let failing_request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("should allow request");

        failing_request.failure(now).expect("should record failure");
        assert_eq!(gate.state(), Ok(CircuitState::Open));

        let probe_time = now + Duration::from_secs(5);
        let probe = gate
            .acquire(probe_time)
            .expect("gate should be usable")
            .expect("should allow probe");

        assert_eq!(gate.state(), Ok(CircuitState::HalfOpen));
        assert!(
            gate.acquire(probe_time)
                .expect("gate should be usable")
                .is_none()
        );

        // 旧请求现在成功，也不能代替本轮探测完成。
        old_request
            .success(probe_time)
            .expect("should ignore old result without error");
        assert_eq!(gate.state(), Ok(CircuitState::HalfOpen));

        probe
            .success(probe_time)
            .expect("should record probe success");
        assert_eq!(gate.state(), Ok(CircuitState::Closed));
    }

    #[test]
    fn dropping_normal_request_does_not_count_as_failure() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(1, Duration::from_secs(5)).expect("valid configuration"));

        let request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("should allow request");
        drop(request);

        assert_eq!(gate.state(), Ok(CircuitState::Closed));
        assert!(gate.acquire(now).expect("gate should be usable").is_some());
    }

    #[test]
    fn dropping_probe_reopens_circuit() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(1, Duration::from_secs(5)).expect("valid configuration"));

        let request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("should allow request");
        request.failure(now).expect("should record failure");

        let probe_time = now + Duration::from_secs(5);
        let probe = gate
            .acquire(probe_time)
            .expect("gate should be usable")
            .expect("should allow probe");

        assert_eq!(gate.state(), Ok(CircuitState::HalfOpen));

        // 模拟取消：没有调用 success 或 failure，直接销毁凭证。
        // Drop 使用真实当前时间开始新的冷却；本测试只验证状态转换。
        drop(probe);

        assert_eq!(gate.state(), Ok(CircuitState::Open));
    }

    #[test]
    fn poisoned_gate_rejects_queries_and_requests() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(1, Duration::from_secs(5)).expect("valid configuration"));

        poison_gate(&gate);

        assert_eq!(gate.state(), Err(CircuitGateError::Poisoned));
        assert_eq!(gate.acquire(now).err(), Some(CircuitGateError::Poisoned),);

        // 没有自动恢复：后续申请仍然失败。
        assert_eq!(gate.acquire(now).err(), Some(CircuitGateError::Poisoned),);
    }

    #[test]
    fn poisoned_gate_rejects_existing_permit_results() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(3, Duration::from_secs(5)).expect("valid configuration"));

        let successful_request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("request should be allowed");

        let failed_request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("request should be allowed");

        poison_gate(&gate);

        assert_eq!(
            successful_request.success(now),
            Err(CircuitGateError::Poisoned),
        );
        assert_eq!(failed_request.failure(now), Err(CircuitGateError::Poisoned),);
    }

    #[test]
    fn dropping_probe_does_not_panic_when_gate_is_poisoned() {
        let now = Instant::now();
        let gate =
            Arc::new(CircuitGate::new(1, Duration::from_secs(5)).expect("valid configuration"));

        let request = gate
            .acquire(now)
            .expect("gate should be usable")
            .expect("request should be allowed");
        request.failure(now).expect("should open circuit");

        let probe = gate
            .acquire(now + Duration::from_secs(5))
            .expect("gate should be usable")
            .expect("probe should be allowed after cooldown");
        assert_eq!(gate.state(), Ok(CircuitState::HalfOpen));

        poison_gate(&gate);

        // 仅用于测试：观察 Drop 是否意外 panic。
        let result = std::panic::catch_unwind(move || {
            drop(probe);
        });

        assert!(result.is_ok(), "dropping the permit should not panic");

        // Drop 没有清除毒锁，也没有假装修复。
        assert_eq!(gate.state(), Err(CircuitGateError::Poisoned));
        assert_eq!(gate.acquire(now).err(), Some(CircuitGateError::Poisoned));
    }
}
