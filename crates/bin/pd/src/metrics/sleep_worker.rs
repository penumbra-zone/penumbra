//! A sleep worker.
//!
//! ### Overview
//!
//! This submodule defines a metric, and an accompanying worker task, for use in measuring
//! scheduler latency in the tokio runtime. This worker will repeatedly sleep for one second, and
//! then observe the amount of time it *actually* spent waiting to be woken up. This is useful for
//! detecting when the asynchronous runtime is being disrupted by blocking I/O, or other expensive
//! non-coöperative computation.
//!
//! Use [`register_metrics()`] to register the [`SLEEP_DRIFT`] metric with an exporter, and spawn
//! the worker onto a runtime by calling [`run()`].

use {
    super::*,
    std::time::{Duration, Instant},
    tokio::time::sleep,
};

pub const SLEEP_DRIFT: &str = "pd_async_sleep_drift_microseconds";

const ONE_SECOND: Duration = Duration::from_secs(1);
const ONE_SECOND_US: u128 = ONE_SECOND.as_micros();

pub fn register_metrics() {
    describe_counter!(
        SLEEP_DRIFT,
        Unit::Microseconds,
        "Tracks drift in the async runtime's timer, in microseconds."
    );
}

/// Run the sleep worker.
///
/// This function will never return.
pub async fn run() -> std::convert::Infallible {
    let counter = counter!(SLEEP_DRIFT);

    loop {
        // Ask the async runtime to pause this task for one second, and then observe the amount of
        // microseconds we were actually suspended.
        let start = Instant::now();
        sleep(ONE_SECOND).await;
        let end = Instant::now();
        let actual = end.duration_since(start).as_micros();

        // Find the difference between the observed sleep duration and our expected duration.
        let drift: u64 = actual
            .saturating_sub(ONE_SECOND_US)
            .try_into()
            .unwrap_or_else(|error| {
                // In the unlikely event that the number of microseconds we waited can't fit into
                // a u64, round down to u64::MAX. This is lossy, but will still indicate that
                // there is a severe issue with the runtime.
                tracing::error!(?error, %actual, "failed to convert timer drift into a u64");
                u64::MAX
            });

        // If there was scheduler drift, increment the counter.
        match drift {
            0 => continue,
            n => counter.increment(n),
        }
    }
}
