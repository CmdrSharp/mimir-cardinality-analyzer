use metrics::histogram;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use std::time::Instant;

pub mod analysis;
pub mod external;
pub mod http;
pub mod process;

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

    // Register metrics
    analysis::register_metrics();
    external::register_metrics();
    http::register_metrics();
    process::register_metrics();
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

#[derive(Debug, Clone)]
pub enum Status {
    Success,
    Failure,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Success => write!(f, "success"),
            Status::Failure => write!(f, "failure"),
        }
    }
}
