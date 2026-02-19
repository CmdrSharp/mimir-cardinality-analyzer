use metrics::{describe_gauge, gauge};

/// Register the metrics for the application
pub(super) fn register_metrics() {
    // Uptime of the application in seconds
    describe_gauge!(
        "process_start_time_seconds",
        "Start time of the process in seconds since the Unix epoch"
    );

    // Build information of the application
    describe_gauge!(
        "build_info",
        "Build information of the application, labeled by version and commit"
    );

    record_process_start_time();
    record_build_info();
}

/// Record the process start time in seconds since the Unix epoch
pub fn record_process_start_time() {
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();

    gauge!("process_start_time_seconds").set(start_time);
}

/// Record the build information of the application
pub fn record_build_info() {
    let version = env!("CARGO_PKG_VERSION");

    gauge!("build_info", "version" => version.to_string()).set(1.0);
}
