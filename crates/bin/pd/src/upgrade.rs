use std::{path::PathBuf, time::Duration};

use penumbra_chain::{
    component::{AppHash, StateReadExt},
    genesis::AppState,
};
use penumbra_stake::StateReadExt as _;
use penumbra_storage::{StateDelta, StateRead, StateWrite, Storage};

use crate::testnet::generate::TestnetConfig;

pub enum Upgrade {
    /// No-op migration
    Noop,
    /// Testnet 60 migration
    Testnet60,
}

pub async fn migrate(path_to_export: PathBuf, upgrade: Upgrade) -> anyhow::Result<()> {
    match upgrade {
        Upgrade::Noop => (),
        Upgrade::Testnet60 => {
            let mut db_path = path_to_export.clone();
            db_path.push("rocksdb");
            let storage = Storage::load(db_path).await?;
            let export_state = storage.latest_snapshot();
            let root_hash = export_state.root_hash().await.expect("can get root hash");
            let app_hash_pre_migration: AppHash = root_hash.into();

            /* --------- writing to the jmt  ------------ */
            tracing::info!(?app_hash_pre_migration, "app hash pre upgrade");
            let mut delta = StateDelta::new(export_state);
            delta.put_raw("testnet_60_forked".to_string(), "done".into());
            let root_hash = storage.commit_in_place(delta).await?;
            let app_hash_post_migration: AppHash = root_hash.into();
            tracing::info!(?app_hash_post_migration, "app hash post upgrade");

            /* --------- collecting genesis data -------- */
            tracing::info!("generating genesis");
            let migrated_state = storage.latest_snapshot();
            let root_hash = migrated_state.root_hash().await.expect("can get root hash");
            let app_hash: AppHash = root_hash.into();
            tracing::info!(?root_hash, "root hash post upgrade2");
            let height = migrated_state
                .get_block_height()
                .await
                .expect("can get block height");
            let next_height = height + 1;
            let chain_params = migrated_state
                .get_chain_params()
                .await
                .expect("can get chain params");

            /* ---------- genereate genesis ------------  */
            let validators = migrated_state.validator_list().await?;
            let mut app_state = AppState::default();
            app_state.chain_params = chain_params;
            app_state.validators = validators.into_iter().map(Into::into).collect();
            let mut genesis = TestnetConfig::make_genesis(app_state.clone()).unwrap();
            genesis.app_hash = app_hash
                .0
                .to_vec()
                .try_into()
                .expect("infaillible conversion");
            genesis.initial_height = next_height as i64;
            genesis.genesis_time = tendermint::time::Time::now();

            let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
            tracing::info!("genesis: {}", genesis_json);
            let mut genesis_path = path_to_export.clone();
            genesis_path.push("genesis.json");
            std::fs::write(genesis_path, genesis_json).expect("can write genesis");
        }
    }
    Ok(())
}
