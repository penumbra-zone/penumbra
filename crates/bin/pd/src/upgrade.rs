use std::path::PathBuf;

use jmt::RootHash;
use penumbra_app::{genesis, SUBSTORE_PREFIXES};
use penumbra_chain::{
    component::{StateReadExt, StateWriteExt},
    genesis::Content as ChainContent,
};
use penumbra_stake::{genesis::Content as StakeContent, StateReadExt as _};
use penumbra_storage::{StateDelta, StateWrite, Storage};

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
            let storage = Storage::load(db_path, SUBSTORE_PREFIXES.to_vec()).await?;
            let export_state = storage.latest_snapshot();
            let root_hash = export_state.root_hash().await.expect("can get root hash");
            let app_hash_pre_migration: RootHash = root_hash.into();
            let height = export_state
                .get_block_height()
                .await
                .expect("can get block height");
            let post_ugprade_height = height.wrapping_add(1);

            /* --------- writing to the jmt  ------------ */
            tracing::info!(?app_hash_pre_migration, "app hash pre upgrade");
            let mut delta = StateDelta::new(export_state);
            delta.put_raw("testnet_60_forked".to_string(), "done".into());
            delta.put_block_height(0u64);
            let root_hash = storage.commit_in_place(delta).await?;
            let app_hash_post_migration: RootHash = root_hash.into();
            tracing::info!(?app_hash_post_migration, "app hash post upgrade");

            /* --------- collecting genesis data -------- */
            tracing::info!("generating genesis");
            let migrated_state = storage.latest_snapshot();
            let root_hash = migrated_state.root_hash().await.expect("can get root hash");
            let app_hash: RootHash = root_hash.into();
            tracing::info!(?root_hash, "root hash post upgrade2");
            let chain_params = migrated_state
                .get_chain_params()
                .await
                .expect("can get chain params");

            /* ---------- genereate genesis ------------  */
            let validators = migrated_state.validator_list().await?;
            let app_state = genesis::Content {
                chain_content: ChainContent { chain_params },
                stake_content: StakeContent {
                    validators: validators.into_iter().map(Into::into).collect(),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut genesis =
                TestnetConfig::make_genesis(app_state.clone()).expect("can make genesis");
            genesis.app_hash = app_hash
                .0
                .to_vec()
                .try_into()
                .expect("infaillible conversion");
            genesis.initial_height = post_ugprade_height as i64;
            genesis.genesis_time = tendermint::time::Time::now();
            let checkpoint = [32u8; 32].to_vec();
            let genesis = TestnetConfig::make_checkpoint(genesis, Some(checkpoint));

            let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
            tracing::info!("genesis: {}", genesis_json);
            let mut genesis_path = path_to_export.clone();
            genesis_path.push("genesis.json");
            std::fs::write(genesis_path, genesis_json).expect("can write genesis");
        }
    }
    Ok(())
}
