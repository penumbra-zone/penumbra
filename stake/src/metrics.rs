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
    register_gauge!(MISSED_BLOCKS);
    describe_gauge!(
        MISSED_BLOCKS,
        Unit::Count,
        "The number of missed blocks per validator"
    );
    register_gauge!(ACTIVE_VALIDATORS);
    describe_gauge!(
        ACTIVE_VALIDATORS,
        Unit::Count,
        "The number of active validators"
    );
    register_gauge!(INACTIVE_VALIDATORS);
    describe_gauge!(
        INACTIVE_VALIDATORS,
        Unit::Count,
        "The number of inactive validators"
    );
    register_gauge!(JAILED_VALIDATORS);
    describe_gauge!(
        JAILED_VALIDATORS,
        Unit::Count,
        "The number of jailed validators"
    );
    register_gauge!(DISABLED_VALIDATORS);
    describe_gauge!(
        DISABLED_VALIDATORS,
        Unit::Count,
        "The number of disabled validators"
    );
    register_gauge!(TOMBSTONED_VALIDATORS);
    describe_gauge!(
        TOMBSTONED_VALIDATORS,
        Unit::Count,
        "The number of tombstoned validators"
    );
}

pub const MISSED_BLOCKS: &str = "penumbra_stake_missed_blocks";
pub const ACTIVE_VALIDATORS: &str = "penumbra_stake_validators_active";
pub const DISABLED_VALIDATORS: &str = "penumbra_stake_validators_disabled";
pub const INACTIVE_VALIDATORS: &str = "penumbra_stake_validators_inactive";
pub const JAILED_VALIDATORS: &str = "penumbra_stake_validators_jailed";
pub const TOMBSTONED_VALIDATORS: &str = "penumbra_stake_validators_tombstoned";
