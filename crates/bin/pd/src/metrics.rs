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

#[allow(unused_imports)] // It is okay if this reëxport isn't used, see above.
pub use metrics::*;

pub mod cpu_worker;
pub mod sleep_worker;

/// Registers all metrics used by this crate.
///
/// For this implementation, in the `pd` crate, we also call the `register_metrics()`
/// functions in our dependencies.
pub fn register_metrics() {
    // This will register metrics for all components.
    penumbra_sdk_app::register_metrics();
    self::sleep_worker::register_metrics();
    self::cpu_worker::register_metrics();
}
