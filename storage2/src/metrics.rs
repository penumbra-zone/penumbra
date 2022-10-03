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
    register_gauge!(TCT_SIZE_BYTES);
    describe_gauge!(
        TCT_SIZE_BYTES,
        Unit::Bytes,
        "The size of the serialized TCT in bytes"
    );
}

// TODO: rename after we remove the old storage crate
pub const TCT_SIZE_BYTES: &str = "penumbra_storage2_tct_size_bytes";
