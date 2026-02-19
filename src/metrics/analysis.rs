use crate::metrics::Status;
use metrics::{counter, describe_counter, describe_gauge, gauge};

/// Register the metrics for the application
pub(super) fn register_metrics() {
    // Count of failures. Should be labeled with the task name and tenant.
    describe_counter!("analysis_errors_total", "Total number of analysis errors");

    // Count of analysis cycles. Should be labeled with the status (success or failure).
    describe_counter!("analysis_cycles_total", "Total number of analysis cycles");

    // Number of tenants discovered.
    describe_gauge!(
        "tenants_discovered_total",
        "Total number of tenants discovered during tenant discovery"
    );

    // Timestamp of the last successful analysis cycle
    describe_gauge!(
        "last_successful_analysis_timestamp",
        "Timestamp of the last successful analysis cycle"
    );

    // Gauge to track whether a given metric is active (1) or inactive (0). Should be labeled with the metric name and tenant.
    describe_gauge!(
        "metric_active",
        "Tracks whether a given metric is active (1) or inactive (0)"
    );
}

/// Record analysis error for a given task and tenant
pub fn record_analysis_error(failure: TaskFailure) {
    match failure {
        TaskFailure::Cycle => counter!("analysis_errors_total", "task" => "cycle").increment(1),
        TaskFailure::Tenant(tenant_id) => {
            counter!("analysis_errors_total", "task" => "tenant", "tenant" => tenant_id)
                .increment(1)
        }
    }
}

/// Record an analysis cycle with the given status
pub fn record_analysis_cycle(status: Status) {
    counter!("analysis_cycles_total", "status" => status.to_string()).increment(1);
}

/// Record the number of tenants discovered
pub fn record_tenants_discovered(count: u64) {
    gauge!("tenants_discovered_total").set(count as f64);
}

/// Record the timestamp of the last successful analysis cycle
pub fn record_successful_analysis() {
    let timestamp = chrono::Utc::now().timestamp() as f64;

    gauge!("last_successful_analysis_timestamp").set(timestamp);
}

/// Create usage metric for a given metric name
pub fn set_metric(metric_name: &str, tenant_id: &str, active: bool) {
    gauge!("metric_active", "metric" => metric_name.to_string(), "tenant" => tenant_id.to_string())
        .set(if active { 1 } else { 0 });
}

#[derive(Debug, Clone)]
pub enum TaskFailure {
    Cycle,
    Tenant(String),
}

impl std::fmt::Display for TaskFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskFailure::Cycle => write!(f, "cycle"),
            TaskFailure::Tenant(_) => write!(f, "tenant"),
        }
    }
}
