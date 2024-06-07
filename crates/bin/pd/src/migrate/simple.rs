//! An example migration script.

use anyhow;
use cnidarium::{StateDelta, StateWrite, Storage};
use jmt::RootHash;
use penumbra_sct::component::clock::EpochManager;
use std::time::Duration;

pub async fn migrate(storage: Storage) -> anyhow::Result<Duration> {
    let export_state = storage.latest_snapshot();
    let root_hash = export_state.root_hash().await.expect("can get root hash");
    let app_hash_pre_migration: RootHash = root_hash.into();

    /* --------- writing to the jmt  ------------ */
    tracing::info!(?app_hash_pre_migration, "app hash pre-upgrade");
    let mut delta = StateDelta::new(export_state);
    let start_time = std::time::SystemTime::now();
    delta.put_raw(
        "banana".to_string(),
        "a good fruit (and migration works!)".into(),
    );
    delta.put_block_height(0u64);

    storage.commit_in_place(delta).await?;

    Ok(start_time.elapsed().expect("start time not set"))
}
