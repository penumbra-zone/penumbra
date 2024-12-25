//! A migration script to reset the chain's halt bit.

use anyhow;
use cnidarium::{StateDelta, Storage};
use penumbra_sdk_governance::StateWriteExt as _;
use std::path::PathBuf;

pub async fn migrate(
    storage: Storage,
    _path_to_export: PathBuf,
    _genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    let export_state = storage.latest_snapshot();
    let mut delta = StateDelta::new(export_state);
    delta.ready_to_start();
    let _ = storage.commit_in_place(delta).await?;
    storage.release().await;
    tracing::info!("migration completed: halt bit is turned off, chain is ready to start");

    Ok(())
}
