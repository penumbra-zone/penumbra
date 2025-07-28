use anyhow::Result;
use cnidarium::StateDelta;

use super::framework::Migration;

pub struct Mainnet5Migration;

impl Migration for Mainnet5Migration {
    fn name(&self) -> &'static str {
        "mainnet-5"
    }

    fn target_app_version(&self) -> u64 {
        12
    }

    async fn migrate_inner(&self, _delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        tracing::info!("Mainnet5 migration is a no-op");
        Ok(())
    }
}
