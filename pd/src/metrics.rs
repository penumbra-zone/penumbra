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
///
/// For this implementation, in the `pd` crate, we also call the `register_metrics()`
/// functions in our dependencies.
pub fn register_metrics() {
    penumbra_storage::register_metrics();
    penumbra_app::stake::register_metrics();
    // penumbra_app::ibc::register_metrics();
    penumbra_shielded_pool::component::register_metrics();

    register_counter!(MEMPOOL_CHECKTX_TOTAL);
    describe_counter!(
        MEMPOOL_CHECKTX_TOTAL,
        Unit::Count,
        "The total number of checktx requests made to the mempool"
    );

    register_gauge!(CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS);
    describe_gauge!(
        CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS,
        Unit::Count,
        "The number of active connections streaming compact blocks"
    );

    register_counter!(CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL);
    describe_counter!(
        CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL,
        Unit::Count,
        "The total number of compact blocks served to clients"
    );
}

pub const MEMPOOL_CHECKTX_TOTAL: &str = "penumbra_pd_mempool_checktx_total";

pub const CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS: &str =
    "penumbra_pd_oblivious_client_compact_active_connections";

pub const CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL: &str =
    "penumbra_pd_oblivious_client_compact_block_served_total";
