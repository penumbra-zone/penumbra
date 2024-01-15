//! Crate-specific metrics functionality.
//!
//! This module re-exports the contents of the `metrics` crate.  This is
//! effectively a way to monkey-patch the functions in this module into the
//! `metrics` crate, at least from the point of view of the other code in this
//! crate.
//!
//! Code in this crate that wants to use metrics should `use crate::metrics;`,
//! so that this module shadows the `metrics` crate.
//!
//! This trick is probably good to avoid in general, because it could be
//! confusing, but in this limited case, it seems like a clean option.

pub use metrics::*;

/// Registers all metrics used by this crate.
pub fn register_metrics() {
    register_histogram!(DEX_ARB_DURATION);
    describe_histogram!(
        DEX_ARB_DURATION,
        Unit::Seconds,
        "The time spent computing arbitrage during endblock phase"
    );
    register_histogram!(DEX_BATCH_DURATION);
    describe_histogram!(
        DEX_BATCH_DURATION,
        Unit::Seconds,
        "The time spent executing batches within the DEX"
    );
    register_histogram!(DEX_PATH_SEARCH_DURATION);
    describe_histogram!(
        DEX_PATH_SEARCH_DURATION,
        Unit::Seconds,
        "The time spent searching for paths while executing trades within the DEX"
    );
    register_histogram!(DEX_ROUTE_FILL_DURATION);
    describe_histogram!(
        DEX_ROUTE_FILL_DURATION,
        Unit::Seconds,
        "The time spent filling routes while executing trades within the DEX"
    );
    register_histogram!(DEX_SWAP_DURATION);
    describe_histogram!(
        DEX_SWAP_DURATION,
        Unit::Seconds,
        "The time spent processing swaps within the DEX"
    );
}

// We configure buckets for the DEX routing times manually, in order to ensure
// Prometheus metrics are structured as a Histogram, rather than as a Summary.
// These values are loosely based on the initial Summary output, and may need to be
// updated over time.
pub const DEX_BUCKETS: &[f64; 5] = &[0.00001, 0.0001, 0.001, 0.01, 0.1];

pub const DEX_PATH_SEARCH_DURATION: &str = "penumbra_dex_path_search_duration_seconds";
pub const DEX_ROUTE_FILL_DURATION: &str = "penumbra_dex_route_fill_duration_seconds";
pub const DEX_ARB_DURATION: &str = "penumbra_dex_arb_duration_seconds";
pub const DEX_BATCH_DURATION: &str = "penumbra_dex_batch_duration_seconds";
pub const DEX_SWAP_DURATION: &str = "penumbra_dex_swap_duration_seconds";
