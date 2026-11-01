//! 进程内指标：响应头就绪与流式正文结束分别采样，不把两者混成同一批样本。

use axum::http::StatusCode;
use serde::Serialize;
use std::{sync::Mutex, time::Duration};

#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamEnd {
    Complete,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct MetricsSnapshot {
    pub responses_ready_total: u64,
    pub http_client_errors_total: u64,
    pub http_server_errors_total: u64,
    pub response_ready_duration_seconds_sum: f64,
    pub streams_ended_total: u64,
    pub stream_body_failures_total: u64,
    pub streams_cancelled_total: u64,
    pub stream_body_duration_seconds_sum: f64,
}

// Option 避免重复声明所有零值；第一次记录时才初始化快照。
static METRICS: Mutex<Option<MetricsSnapshot>> = Mutex::new(None);

impl MetricsSnapshot {
    fn record_response(&mut self, status: StatusCode, elapsed: Duration) {
        self.responses_ready_total += 1;
        self.response_ready_duration_seconds_sum += elapsed.as_secs_f64();
        if status.is_client_error() {
            self.http_client_errors_total += 1;
        }
        if status.is_server_error() {
            self.http_server_errors_total += 1;
        }
    }

    fn record_stream(&mut self, end: StreamEnd, elapsed: Duration) {
        self.streams_ended_total += 1;
        self.stream_body_duration_seconds_sum += elapsed.as_secs_f64();
        match end {
            StreamEnd::Complete => {}
            StreamEnd::Failed => self.stream_body_failures_total += 1,
            StreamEnd::Cancelled => self.streams_cancelled_total += 1,
        }
    }
}

pub(crate) fn record_response(status: StatusCode, elapsed: Duration) {
    let mut metrics = METRICS.lock().expect("metrics lock should not be poisoned");
    metrics
        .get_or_insert_with(MetricsSnapshot::default)
        .record_response(status, elapsed);
}

pub(crate) fn record_stream(end: StreamEnd, elapsed: Duration) {
    let mut metrics = METRICS.lock().expect("metrics lock should not be poisoned");
    metrics
        .get_or_insert_with(MetricsSnapshot::default)
        .record_stream(end, elapsed);
}

pub(crate) fn snapshot() -> MetricsSnapshot {
    METRICS
        .lock()
        .expect("metrics lock should not be poisoned")
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(200, 0, 0)]
    #[case(399, 0, 0)]
    #[case(400, 1, 0)]
    #[case(499, 1, 0)]
    #[case(500, 0, 1)]
    #[case(599, 0, 1)]
    fn classifies_http_error_boundaries(
        #[case] status: u16,
        #[case] client: u64,
        #[case] server: u64,
    ) {
        let mut metrics = MetricsSnapshot::default();
        metrics.record_response(
            StatusCode::from_u16(status).unwrap(),
            Duration::from_millis(250),
        );
        assert_eq!(metrics.responses_ready_total, 1);
        assert_eq!(metrics.http_client_errors_total, client);
        assert_eq!(metrics.http_server_errors_total, server);
        assert_eq!(metrics.response_ready_duration_seconds_sum, 0.25);
    }

    #[test]
    fn separates_http_status_from_stream_failure_and_cancellation() {
        let mut metrics = MetricsSnapshot::default();
        for status in [
            StatusCode::OK,
            StatusCode::OK,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            metrics.record_response(status, Duration::from_millis(250));
        }
        // 200 正文失败、200 中途取消、完整读完 503 是三种不同结果。
        for end in [StreamEnd::Failed, StreamEnd::Cancelled, StreamEnd::Complete] {
            metrics.record_stream(end, Duration::from_millis(500));
        }
        assert_eq!(metrics.responses_ready_total, 3);
        assert_eq!(metrics.http_server_errors_total, 1);
        assert_eq!(metrics.http_client_errors_total, 0);
        assert_eq!(metrics.streams_ended_total, 3);
        assert_eq!(metrics.stream_body_failures_total, 1);
        assert_eq!(metrics.streams_cancelled_total, 1);
        assert_eq!(metrics.response_ready_duration_seconds_sum, 0.75);
        assert_eq!(metrics.stream_body_duration_seconds_sum, 1.5);
    }
}
