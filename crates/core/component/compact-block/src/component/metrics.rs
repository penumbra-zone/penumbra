//! Crate-specific metrics functionality.
//!
//! This module re-exports the contents of the `metrics` crate.  This is
//! effectively a way to monkey-patch the functions in this module into the
//! `metrics` crate, at least from the point of view of the other code in this
//! crate.
//!
//! Code in this crate that wants to use metrics should `use crate::component::metrics;`,
//! so that this module shadows the `metrics` crate.
//!
//! This trick is probably good to avoid in general, because it could be
//! confusing, but in this limited case, it seems like a clean option.

pub use metrics::*;

/// Registers all metrics used by this crate.
///
/// The source code contains the metrics descriptions.
pub fn register_metrics() {
    register_gauge!(COMPACT_BLOCK_RANGE_ACTIVE_CONNECTIONS);
    describe_gauge!(
        COMPACT_BLOCK_RANGE_ACTIVE_CONNECTIONS,
        Unit::Count,
        "The number of active connections streaming compact blocks"
    );

    register_counter!(COMPACT_BLOCK_RANGE_SERVED_TOTAL);
    describe_counter!(
        COMPACT_BLOCK_RANGE_SERVED_TOTAL,
        Unit::Count,
        "The total number of compact blocks served to clients"
    );
}

// Sample code for reference -- delete when adding the first metric
// pub const MEMPOOL_CHECKTX_TOTAL: &str = "penumbra_pd_mempool_checktx_total";

pub const COMPACT_BLOCK_RANGE_ACTIVE_CONNECTIONS: &str =
    "penumbra_component_compact_block_compact_block_range_active_connections";

pub const COMPACT_BLOCK_RANGE_SERVED_TOTAL: &str =
    "penumbra_component_compact_block_compact_block_range_served_total";
