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
    describe_counter!(
        DEX_PATH_SEARCH_RELAX_PATH_DURATION,
        Unit::Seconds,
        "The time spent relaxing a path while routing trades within the DEX"
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
// These values are logarithmically spaced from 5us to 67ms.
const GENERIC_DEX_BUCKETS: &[f64; 16] = &[
    0.0005,
    0.000792,
    0.001256,
    0.001991,
    0.003155,
    0.005,
    0.00648985018,
    0.00842363108,
    0.01093362074,
    0.01419151211,
    0.01842015749,
    0.02390881249,
    0.03103292223,
    0.0402798032,
    0.05228197763,
    0.06786044041,
];

pub const DEX_PATH_SEARCH_DURATION: &str = "penumbra_dex_path_search_duration_seconds";
pub const DEX_PATH_SEARCH_RELAX_PATH_DURATION: &str =
    "penumbra_dex_path_search_relax_path_duration_seconds";
pub const DEX_ROUTE_FILL_DURATION: &str = "penumbra_dex_route_fill_duration_seconds";
pub const DEX_ARB_DURATION: &str = "penumbra_dex_arb_duration_seconds";
pub const DEX_BATCH_DURATION: &str = "penumbra_dex_batch_duration_seconds";
pub const DEX_RPC_SIMULATE_TRADE_DURATION: &str =
    "penumbra_dex_rpc_simulate_trade_duration_seconds";

/// An extension trait providing DEX-related interfaces for [`PrometheusBuilder`].
///
/// [builder]: metrics_exporter_prometheus::PrometheusBuilder
pub trait PrometheusBuilderExt
where
    Self: Sized,
{
    /// Configure buckets for histogram metrics.
    fn set_buckets_for_dex_metrics(self) -> Result<Self, metrics_exporter_prometheus::BuildError>;
}

impl PrometheusBuilderExt for metrics_exporter_prometheus::PrometheusBuilder {
    fn set_buckets_for_dex_metrics(self) -> Result<Self, metrics_exporter_prometheus::BuildError> {
        use metrics_exporter_prometheus::Matcher::Full;
        self.set_buckets_for_metric(
            Full(DEX_PATH_SEARCH_DURATION.to_owned()),
            GENERIC_DEX_BUCKETS,
        )?
        .set_buckets_for_metric(
            Full(DEX_PATH_SEARCH_RELAX_PATH_DURATION.to_owned()),
            GENERIC_DEX_BUCKETS,
        )?
        .set_buckets_for_metric(
            Full(DEX_ROUTE_FILL_DURATION.to_owned()),
            GENERIC_DEX_BUCKETS,
        )?
        .set_buckets_for_metric(Full(DEX_ARB_DURATION.to_owned()), GENERIC_DEX_BUCKETS)?
        .set_buckets_for_metric(Full(DEX_BATCH_DURATION.to_owned()), GENERIC_DEX_BUCKETS)?
        .set_buckets_for_metric(
            Full(DEX_RPC_SIMULATE_TRADE_DURATION.to_owned()),
            GENERIC_DEX_BUCKETS,
        )
    }
}
