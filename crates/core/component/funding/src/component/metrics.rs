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
    describe_gauge!(
        TOTAL_VALIDATOR_REWARDS,
        Unit::Count,
        "The total amount of rewards distributed to validators during the epoch"
    );

    describe_gauge!(
        VALIDATOR_FUNDING_VS_BUDGET_DIFFERENCE,
        Unit::Count,
        "The delta between the total amount of rewards distributed to validators and the rewards budget for the epoch"
    );

    describe_histogram!(
        TOTAL_FUNDING_STREAMS_PROCESSING_TIME,
        Unit::Milliseconds,
        "The amount of time spent processing funding rewards for the epoch"
    );

    describe_histogram!(
        FETCH_FUNDING_QUEUE_LATENCY,
        Unit::Milliseconds,
        "The amount of time spent fetching the funding queue from storage"
    );
}

pub const TOTAL_VALIDATOR_REWARDS: &str = "penumbra_funding_total_validator_rewards_staking_token";
pub const VALIDATOR_FUNDING_VS_BUDGET_DIFFERENCE: &str =
    "penumbra_funding_validator_vs_budget_difference_staking_token";
pub const FETCH_FUNDING_QUEUE_LATENCY: &str =
    "penumbra_funding_fetch_funding_queue_latency_milliseconds";
pub const TOTAL_FUNDING_STREAMS_PROCESSING_TIME: &str =
    "penumbra_funding_streams_total_processing_time_milliseconds";
