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

#[allow(unused_imports)]
pub use metrics::*;

/// Registers all metrics used by this crate.
pub fn register_metrics() {
    /*
    // Sample code for reference -- delete when adding the first metric
    register_counter!(MEMPOOL_CHECKTX_TOTAL);
    describe_counter!(
        MEMPOOL_CHECKTX_TOTAL,
        "The total number of checktx requests made to the mempool"
    );
     */
}

// Sample code for reference -- delete when adding the first metric
// pub const MEMPOOL_CHECKTX_TOTAL: &str = "penumbra_pd_mempool_checktx_total";
