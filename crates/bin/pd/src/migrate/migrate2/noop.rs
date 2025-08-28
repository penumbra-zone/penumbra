//! A no-op migration that resets the halt bit and produces a new genesis without changes.
//! This is useful for all hard-forks that do not require a migration, but break consensus.

use anyhow::Result;
use cnidarium::StateDelta;

use super::Migration;

/// A migration that performs no changes to the state, except resetting the halt bit.
/// Creates a different new chain, with a refreshed genesis state.
pub struct NoOpMigration {
    target_app_version: Option<u64>,
}

impl NoOpMigration {
    /// Create a new NoOp migration with an optional target app version.
    pub fn new(target_app_version: Option<u64>) -> Self {
        Self { target_app_version }
    }
}

impl Migration for NoOpMigration {
    fn name(&self) -> &'static str {
        "no-op"
    }

    fn target_app_version(&self) -> Option<u64> {
        // By default, this target no specific versions, but
        // it is possible to specify a target app version
        // via a flag. In that case, UIP-4 applies, and version
        // migration will occur.
        self.target_app_version
    }

    async fn migrate_inner(&self, _delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        // No changes to state - the framework handles resetting the halt bit
        Ok(())
    }
}
