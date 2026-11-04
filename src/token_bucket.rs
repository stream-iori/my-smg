//! 按整数时间结算的令牌桶。

use std::time::{Duration, Instant};

/// 整数令牌桶；每个补充间隔增加一枚，空闲累计不超过容量。
#[derive(Debug)]
pub struct TokenBucket {
    //总容量
    capacity: usize,

    //当前剩余多少枚
    tokens: usize,

    //多久补充一枚
    refill_interval: Duration,

    //已经结算到哪个时间点
    last_refill: Instant,
}

impl TokenBucket {
    /// 创建初始满桶；容量与补充间隔必须大于零，now 为统一时间基准。
    pub fn try_new(
        capacity: usize,
        refill_interval: Duration,
        now: Instant,
    ) -> Result<Self, &'static str> {
        if capacity == 0 {
            return Err("capacity must be greater than zero");
        }

        if refill_interval.is_zero() {
            return Err("refill interval must be greater than zero");
        }

        Ok(Self {
            capacity,
            tokens: capacity,
            refill_interval,
            last_refill: now,
        })
    }

    fn refill(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last_refill);

        // as_nanos() 返回 u128，保留整数时间精度。
        let interval_nanos = self.refill_interval.as_nanos();
        let elapsed_nanos = elapsed.as_nanos();

        let refill_count = elapsed_nanos / interval_nanos;
        if refill_count == 0 {
            return;
        }

        // 只补充桶里放得下的数量，避免超过容量。
        let available_space = self.capacity - self.tokens;
        let added = refill_count.min(available_space as u128) as usize;

        self.tokens += added;

        //保留不足一个完整间隔的时间
        let remainder_nanos = elapsed_nanos % interval_nanos;

        //把总纳秒数拆成“整秒 + 不足一秒的纳秒”，构造 Duration
        let remainder = Duration::new(
            (remainder_nanos / 1_000_000_000) as u64,
            (remainder_nanos % 1_000_000_000) as u32,
        );

        // 即使桶已经满了，也要推进时间基准，
        // 防止多余额度留到下一次使用。
        self.last_refill = now - remainder;
    }

    /// 先按时间补充，再尝试扣一枚；失败不透支，早于时间基准不补充。
    pub fn try_acquire(&mut self, now: Instant) -> bool {
        //先结算时间带来的新额度.
        self.refill(now);
        if self.tokens == 0 {
            return false;
        }
        self.tokens -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::TokenBucket;
    use rstest::rstest;
    use std::time::{Duration, Instant};

    fn bucket() -> (Instant, TokenBucket) {
        // 起点与桶一起返回，确保整条时间序列使用同一个时间基准。
        let start = Instant::now();
        let bucket = TokenBucket::try_new(2, Duration::from_millis(100), start)
            .expect("valid configuration");
        (start, bucket)
    }

    #[rstest]
    #[case::zero_capacity(0, Duration::from_millis(100))]
    #[case::zero_interval(2, Duration::ZERO)]
    fn rejects_invalid_configuration(#[case] capacity: usize, #[case] refill_interval: Duration) {
        let start = Instant::now();
        assert!(TokenBucket::try_new(capacity, refill_interval, start).is_err());
    }

    #[test]
    fn limits_requests_and_caps_idle_refill() {
        let (start, mut bucket) = bucket();

        // 按顺序操作同一个桶。
        // 每项：距起点的毫秒数、是否允许。
        let cases = [
            (0, true),
            (0, true),
            (0, false),
            (99, false),
            (100, true),
            (100, false),
            (150, false),
            (200, true),
            // 长时间闲置，最多恢复到两枚。
            (10_000, true),
            (10_000, true),
            (10_000, false),
            // 超过容量的额度不能留到下一次。
            (10_050, false),
            (10_100, true),
            (10_100, false),
        ];

        for (elapsed_ms, expected) in cases {
            let now = start + Duration::from_millis(elapsed_ms);

            assert_eq!(
                bucket.try_acquire(now),
                expected,
                "request at {elapsed_ms}ms",
            );
        }
    }

    #[test]
    fn preserves_partial_refill_interval() {
        let (start, mut bucket) = bucket();

        // 先耗尽初始额度。
        assert!(bucket.try_acquire(start));
        assert!(bucket.try_acquire(start));

        // 150ms 时补一枚，取走它，保留余下的 50ms。
        assert!(bucket.try_acquire(start + Duration::from_millis(150)));

        assert!(!bucket.try_acquire(start + Duration::from_millis(199)));

        // 再过 50ms 就足够补下一枚。
        assert!(bucket.try_acquire(start + Duration::from_millis(200)));
    }
}
