#![cfg(feature = "metrics")]
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
        STORAGE_GET_RAW_DURATION,
        Unit::Seconds,
        "The duration of a get_raw request"
    );
    describe_histogram!(
        STORAGE_NONCONSENSUS_GET_RAW_DURATION,
        Unit::Seconds,
        "The duration of a nonverifiable_get_raw request"
    );
}

pub const STORAGE_GET_RAW_DURATION: &str = "cnidarium_get_raw_duration_seconds";
pub const STORAGE_NONCONSENSUS_GET_RAW_DURATION: &str =
    "cnidarium_nonverifiable_get_raw_duration_seconds";
