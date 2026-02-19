use crate::metrics::Status;
use metrics::{counter, describe_counter, describe_histogram, histogram};
use std::time::Instant;

/// Register the metrics for the application
pub(super) fn register_metrics() {
    // Count of failed external requests. Should be labeled with the target.
    describe_counter!(
        "external_request_failures_total",
        "Total number of failed external requests"
    );

    // Latency of external requests in seconds, labeled by the the target and status (success or failure).
    describe_histogram!(
        "external_request_duration_seconds",
        "Duration of external requests in seconds"
    );

    // Number of exections of mimirtool along with the command and status (success or failure).
    describe_counter!(
        "mimirtool_executions_total",
        "Total number of executions of mimirtool"
    );

    // Duration of mimirtool executions in seconds, labeled by the command.
    describe_histogram!(
        "mimirtool_duration_seconds",
        "Duration of mimirtool executions in seconds"
    );
}

/// Record an external request failure for a given target
pub fn record_external_request_failure(target: Target) {
    counter!("external_request_failures_total", "target" => target.to_string()).increment(1);
}

/// Create a timer for an external request to a given target
pub fn external_request_timer(target: Target) -> Timer {
    Timer::new("external_request_duration_seconds").with_label("target", target.to_string())
}

/// Record an execution of mimirtool for a given command and status
pub fn record_mimirtool_execution(command: Command, status: Status) {
    counter!("mimirtool_executions_total", "command" => command.to_string(), "status" => status.to_string()).increment(1);
}

/// Create a timer for mimirtool commands
pub fn mimirtool_timer(command: Command) -> Timer {
    Timer::new("mimirtool_duration_seconds").with_label("command", command.to_string())
}

pub struct Timer {
    metric_name: &'static str,
    start_time: Instant,
    labels: Vec<(String, String)>,
}

impl Timer {
    /// Create a new timer
    pub fn new(metric_name: &'static str) -> Self {
        Self {
            metric_name,
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

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed().as_secs_f64();

        if self.labels.is_empty() {
            histogram!(self.metric_name).record(duration);
        } else {
            let labels: Vec<_> = self
                .labels
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            histogram!(self.metric_name, &labels).record(duration);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    StoreGateway,
    Querier,
    Grafana,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::StoreGateway => write!(f, "store-gateway"),
            Target::Querier => write!(f, "querier"),
            Target::Grafana => write!(f, "grafana"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    AnalyzeGrafana,
    AnalyzePrometheus,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::AnalyzeGrafana => write!(f, "analyze_grafana"),
            Command::AnalyzePrometheus => write!(f, "analyze_prometheus"),
        }
    }
}
