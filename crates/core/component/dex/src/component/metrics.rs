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
    describe_histogram!(
        DEX_ARB_DURATION,
        Unit::Seconds,
        "The time spent computing arbitrage during endblock phase"
    );
    describe_histogram!(
        DEX_BATCH_DURATION,
        Unit::Seconds,
        "The time spent executing batches within the DEX"
    );
    describe_histogram!(
        DEX_PATH_SEARCH_DURATION,
        Unit::Seconds,
        "The time spent searching for paths while executing trades within the DEX"
    );
    describe_histogram!(
        DEX_ROUTE_FILL_DURATION,
        Unit::Seconds,
        "The time spent filling routes while executing trades within the DEX"
    );
    describe_histogram!(
        DEX_RPC_SIMULATE_TRADE_DURATION,
        Unit::Seconds,
        "The time spent processing a SimulateTrade RPC request"
    );
}

// We configure buckets for the DEX routing times manually, in order to ensure
// Prometheus metrics are structured as a Histogram, rather than as a Summary.
// These values may need to be updated over time.
// These values are logarithmically spaced from 5ms to 250ms.
pub const DEX_BUCKETS: &[f64; 16] = &[
    5.,
    6.48985018,
    8.42363108,
    10.93362074,
    14.19151211,
    18.42015749,
    23.90881249,
    31.03292223,
    40.2798032,
    52.28197763,
    67.86044041,
    88.08081833,
    114.32626298,
    148.39206374,
    192.6084524,
    250.,
];

pub const DEX_PATH_SEARCH_DURATION: &str = "penumbra_dex_path_search_duration_seconds";
pub const DEX_ROUTE_FILL_DURATION: &str = "penumbra_dex_route_fill_duration_seconds";
pub const DEX_ARB_DURATION: &str = "penumbra_dex_arb_duration_seconds";
pub const DEX_BATCH_DURATION: &str = "penumbra_dex_batch_duration_seconds";
pub const DEX_RPC_SIMULATE_TRADE_DURATION: &str =
    "penumbra_dex_rpc_simulate_trade_duration_seconds";
