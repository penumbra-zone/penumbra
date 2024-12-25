//! An example migration script.

use anyhow;
use cnidarium::{StateDelta, StateWrite, Storage};
use jmt::RootHash;
use penumbra_sdk_app::app::StateReadExt as _;
use penumbra_sdk_sct::component::clock::{EpochManager, EpochRead};
use std::path::PathBuf;

use crate::network::generate::NetworkConfig;
pub async fn migrate(
    storage: Storage,
    path_to_export: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    let export_state = storage.latest_snapshot();
    let root_hash = export_state.root_hash().await.expect("can get root hash");
    let height = export_state
        .get_block_height()
        .await
        .expect("can get block height");
    let app_hash_pre_migration: RootHash = root_hash.into();
    let post_ugprade_height = height.wrapping_add(1);

    /* --------- writing to the jmt  ------------ */
    tracing::info!(?app_hash_pre_migration, "app hash pre-upgrade");
    let mut delta = StateDelta::new(export_state);
    delta.put_raw(
        "banana".to_string(),
        "a good fruit (and migration works!)".into(),
    );
    delta.put_block_height(0u64);
    let root_hash = storage.commit_in_place(delta).await?;
    let app_hash_post_migration: RootHash = root_hash.into();
    tracing::info!(?app_hash_post_migration, "app hash post upgrade");

    /* --------- collecting genesis data -------- */
    tracing::info!("generating genesis");
    let migrated_state = storage.latest_snapshot();
    let root_hash = migrated_state.root_hash().await.expect("can get root hash");
    let app_hash: RootHash = root_hash.into();
    tracing::info!(?root_hash, "root hash from snapshot (post-upgrade)");

    /* ---------- generate genesis ------------  */
    let chain_id = migrated_state.get_chain_id().await?;
    let app_state = penumbra_sdk_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = NetworkConfig::make_genesis(app_state.clone()).expect("can make genesis");
    genesis.app_hash = app_hash
        .0
        .to_vec()
        .try_into()
        .expect("infaillible conversion");
    genesis.initial_height = post_ugprade_height as i64;
    genesis.genesis_time = genesis_start.unwrap_or_else(|| {
        let now = tendermint::time::Time::now();
        tracing::info!(%now, "no genesis time provided, detecting a testing setup");
        now
    });
    let checkpoint = app_hash.0.to_vec();
    let genesis = NetworkConfig::make_checkpoint(genesis, Some(checkpoint));

    let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
    tracing::info!("genesis: {}", genesis_json);
    let genesis_path = path_to_export.join("genesis.json");
    std::fs::write(genesis_path, genesis_json).expect("can write genesis");

    let validator_state_path = path_to_export.join("priv_validator_state.json");
    let fresh_validator_state = crate::network::generate::NetworkValidator::initial_state();
    std::fs::write(validator_state_path, fresh_validator_state).expect("can write validator state");
    Ok(())
}
