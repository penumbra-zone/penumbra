//! A metrics-focused worker for gathering CPU load and other system stats.
//!
//! ### Overview
//!
//! This submodule provides a worker that wraps the [metrics-process] logic to export OS-level
//! runtime information about `pd`.

use std::time::Duration;
use tokio::time::sleep;

use metrics_process::Collector;

/// The time to sleep between polling the OS for process info about `pd`.
const SLEEP_DURATION: Duration = Duration::from_secs(2);
/// The string prepended to all metrics emitted by [metrics-process].
const METRICS_PREFIX: &str = "pd_";

pub fn register_metrics() {
    // Call `describe()` method to register help string.
    let collector = Collector::new(METRICS_PREFIX);
    collector.describe();
}

/// Run the cpu worker.
///
/// This function will never return.
pub async fn run() -> std::convert::Infallible {
    let collector = Collector::new(METRICS_PREFIX);
    loop {
        // Periodically call `collect()` method to update information.
        collector.collect();
        sleep(SLEEP_DURATION).await;
    }
}
