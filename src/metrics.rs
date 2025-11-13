use metrics::{describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use std::time::Instant;

pub static METRICS_HANDLE: OnceCell<Option<PrometheusHandle>> = OnceCell::new();

/// Register the metrics for the application
pub fn register_metrics() {
    let builder = PrometheusBuilder::new();

    let handle = builder
        .install_recorder()
        .expect("Failed to install recorder");

    METRICS_HANDLE
        .set(Some(handle.clone()))
        .unwrap_or_else(|_| {
            panic!("Failed to set the metrics handle");
        });

    describe_gauge!(
        "metric_active",
        "Tracks whether a given metric is active (1) or inactive (0)"
    );

    describe_histogram!("task_duration_seconds", "Duration of a task in seconds");
}

/// Create usage metric for a given metric name
pub fn set_metric(metric_name: &str, tenant_id: &str, active: bool) {
    gauge!("metric_active", "metric" => metric_name.to_string(), "tenant" => tenant_id.to_string())
        .set(if active { 1 } else { 0 });
}

pub struct Timer {
    start_time: Instant,
    labels: Vec<(String, String)>,
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            labels: Vec::new(),
        }
    }

    /// Add a label to the timer
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed().as_secs_f64();

        if self.labels.is_empty() {
            histogram!("task_duration_seconds").record(duration);
        } else {
            let labels: Vec<_> = self
                .labels
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            histogram!("task_duration_seconds", &labels).record(duration);
        }
    }
}
