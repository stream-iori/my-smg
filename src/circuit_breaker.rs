use std::time::{Duration, Instant};
pub mod gate;

/// Closed：正常放行。
/// Open：拒绝请求，等待冷却。
/// HalfOpen：已经放行一个探测，其他请求暂时拒绝。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    consecutive_failures: usize,
    failure_threshold: usize,

    //冷却持续时间
    cooldown: Duration,

    //熔断触发时间
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, cooldown: Duration) -> Result<Self, &'static str> {
        if failure_threshold == 0 {
            return Err("failure threshold must be greater than zero");
        }

        if cooldown.is_zero() {
            return Err("cooldown must be greater than zero");
        }

        Ok(Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            failure_threshold,
            cooldown,
            opened_at: None,
        })
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    pub fn allow_request(&mut self, now: Instant) -> bool {
        match self.state {
            CircuitState::Closed => true,

            CircuitState::Open => {
                let Some(opened_at) = self.opened_at else {
                    // 状态不完整时，保守地拒绝请求。
                    return false;
                };

                let elapsed = now.saturating_duration_since(opened_at);

                //冷却期间内,不能放行第二个请求
                if elapsed < self.cooldown {
                    return false;
                }

                // 为本次请求预留唯一探测名额。
                // 真正发送请求由调用方负责。
                self.state = CircuitState::HalfOpen;
                true
            }

            // 探测尚未完成，不能放行第二个请求。
            CircuitState::HalfOpen => false,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // 成功则打断连续失败。
                self.consecutive_failures = 0;
            }

            CircuitState::HalfOpen => {
                // 本模型要求：该结果来自刚才放行的探测。
                self.state = CircuitState::Closed;
                self.consecutive_failures = 0;
                self.opened_at = None;
            }

            // 熔断期间，迟到的成功结果不能直接恢复放行。
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&mut self, now: Instant) {
        match self.state {
            CircuitState::Closed => {
                self.consecutive_failures += 1;

                if self.consecutive_failures >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.opened_at = Some(now);
                }
            }

            CircuitState::HalfOpen => {
                // 探测失败，从失败时刻重新计算冷却。
                self.state = CircuitState::Open;
                self.opened_at = Some(now);
            }

            // 不让迟到结果不断延长冷却。
            CircuitState::Open => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CircuitBreaker, CircuitState};
    use std::time::{Duration, Instant};

    #[test]
    fn checks_failure_threshold_cases() {
        // 场景、阈值、失败次数、预期状态、是否放行。
        let cases = [
            ("initial", 3, 0, CircuitState::Closed, true),
            ("below threshold", 3, 2, CircuitState::Closed, true),
            ("at threshold", 3, 3, CircuitState::Open, false),
            ("above threshold", 3, 4, CircuitState::Open, false),
            ("threshold one", 1, 1, CircuitState::Open, false),
        ];

        for (name, threshold, failures, expected_state, expected_allowed) in cases {
            let now = Instant::now();
            let mut breaker = CircuitBreaker::new(threshold, Duration::from_secs(5))
                .expect("valid configuration");

            for _ in 0..failures {
                breaker.record_failure(now);
            }

            assert_eq!(breaker.state(), expected_state, "case: {name}");
            assert_eq!(breaker.allow_request(now), expected_allowed, "case: {name}",);
        }
    }

    #[test]
    fn success_resets_consecutive_failures() {
        let now = Instant::now();
        let mut breaker =
            CircuitBreaker::new(3, Duration::from_secs(5)).expect("valid configuration");

        breaker.record_failure(now);
        breaker.record_failure(now);
        breaker.record_success();

        // 成功清零后，重新累计两次失败仍不足以熔断。
        breaker.record_failure(now);
        breaker.record_failure(now);

        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_request(now));

        breaker.record_failure(now);

        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request(now));
    }

    #[test]
    fn late_results_do_not_reset_cooldown() {
        let now = Instant::now();
        let mut breaker =
            CircuitBreaker::new(1, Duration::from_secs(5)).expect("valid configuration");

        breaker.record_failure(now);

        let later = now + Duration::from_secs(4);
        breaker.record_success();
        breaker.record_failure(later);

        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request(later));

        // 冷却仍从最初熔断时计算。
        assert!(breaker.allow_request(now + Duration::from_secs(5)));
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn allows_only_one_probe_after_cooldown() {
        let cases = [
            ("before cooldown", 4, false, CircuitState::Open),
            ("at cooldown", 5, true, CircuitState::HalfOpen),
            ("after cooldown", 6, true, CircuitState::HalfOpen),
        ];

        for (name, elapsed_secs, expected_allowed, expected_state) in cases {
            let now = Instant::now();
            let mut breaker =
                CircuitBreaker::new(1, Duration::from_secs(5)).expect("valid configuration");

            breaker.record_failure(now);

            let probe_time = now + Duration::from_secs(elapsed_secs);

            assert_eq!(
                breaker.allow_request(probe_time),
                expected_allowed,
                "case: {name}",
            );
            assert_eq!(breaker.state(), expected_state, "case: {name}");

            // 未冷却时拒绝；已放行探测后也拒绝。
            assert!(!breaker.allow_request(probe_time), "case: {name}");
        }
    }

    #[test]
    fn successful_probe_closes_circuit() {
        let now = Instant::now();
        let mut breaker =
            CircuitBreaker::new(1, Duration::from_secs(5)).expect("valid configuration");

        breaker.record_failure(now);

        let probe_time = now + Duration::from_secs(5);
        assert!(breaker.allow_request(probe_time));
        assert!(!breaker.allow_request(probe_time));

        breaker.record_success();

        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_request(probe_time));
    }

    #[test]
    fn failed_probe_starts_new_cooldown() {
        let now = Instant::now();
        let mut breaker =
            CircuitBreaker::new(1, Duration::from_secs(5)).expect("valid configuration");

        breaker.record_failure(now);

        let probe_time = now + Duration::from_secs(5);
        assert!(breaker.allow_request(probe_time));

        // 探测一秒后失败，新冷却从第六秒开始。
        let failed_at = now + Duration::from_secs(6);
        breaker.record_failure(failed_at);

        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request(failed_at));
        assert!(!breaker.allow_request(now + Duration::from_secs(10)));
        assert!(breaker.allow_request(now + Duration::from_secs(11)));
    }

    #[test]
    fn rejects_invalid_configuration() {
        let cases = [
            ("zero threshold", 0, Duration::from_secs(5)),
            ("zero cooldown", 3, Duration::ZERO),
        ];

        for (name, threshold, cooldown) in cases {
            assert!(
                CircuitBreaker::new(threshold, cooldown).is_err(),
                "case: {name}",
            );
        }
    }
}
