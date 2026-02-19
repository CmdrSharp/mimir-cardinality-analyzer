use metrics::{counter, describe_counter, describe_histogram, histogram};
use std::time::Instant;

/// Register the metrics for the application
pub(super) fn register_metrics() {
    // Number of HTTP requests
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests per endpoint"
    );

    // Latency of serving HTTP requests per endpoint
    describe_histogram!(
        "http_request_duration_seconds",
        "Duration of HTTP requests in seconds per endpoint"
    );
}

/// Record an HTTP request for a given endpoint
pub fn record_http_request(endpoint: &str) {
    counter!("http_requests_total", "endpoint" => endpoint.to_string()).increment(1);
}

/// Create a timer for an HTTP request to a given endpoint
pub fn http_request_timer(endpoint: &str) -> Timer {
    Timer::default().with_label("endpoint", endpoint.to_string())
}

pub struct Timer {
    start_time: Instant,
    labels: Vec<(String, String)>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            labels: Vec::new(),
        }
    }
}

impl Timer {
    /// Add a label to the timer
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed().as_secs_f64();

        if self.labels.is_empty() {
            histogram!("http_request_duration_seconds").record(duration);
        } else {
            let labels: Vec<_> = self
                .labels
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            histogram!("http_request_duration_seconds", &labels).record(duration);
        }
    }
}
