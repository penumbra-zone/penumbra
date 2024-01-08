use std::path::PathBuf;

use cnidarium::{StateDelta, StateWrite, Storage};
use jmt::RootHash;
use penumbra_app::{genesis, SUBSTORE_PREFIXES};
use penumbra_chain::{
    component::{StateReadExt, StateWriteExt},
    genesis::Content as ChainContent,
};
use penumbra_stake::{genesis::Content as StakeContent, StateReadExt as _};

use crate::testnet::generate::TestnetConfig;

pub enum Upgrade {
    /// No-op migration
    Noop,
    /// A simple migration: adds a key to the consensus state.
    /// This is useful for testing upgrade mechanisms, including in production.
    SimpleUpgrade,
    /// Migrates from testnet-64 to testnet-65.
    Testnet65,
}

impl Upgrade {
    pub async fn migrate(
        &self,
        path_to_export: PathBuf,
        genesis_start: Option<tendermint::time::Time>,
    ) -> anyhow::Result<()> {
        match self {
            Upgrade::Noop => (),
            Upgrade::SimpleUpgrade => {
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
                tracing::info!(?app_hash_pre_migration, "app hash pre-upgrade");
                let mut delta = StateDelta::new(export_state);
                delta.put_raw("has_migrated".to_string(), "yes".into());
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
                let chain_params = migrated_state
                    .get_chain_params()
                    .await
                    .expect("can get chain params");

                /* ---------- generate genesis ------------  */
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
                genesis.genesis_time = genesis_start.unwrap_or_else(|| {
                    let now = tendermint::time::Time::now();
                    tracing::info!(%now, "no genesis time provided, detecting a testing setup");
                    now
                });
                let checkpoint = app_hash.0.to_vec();
                let genesis = TestnetConfig::make_checkpoint(genesis, Some(checkpoint));

                let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
                tracing::info!("genesis: {}", genesis_json);
                let mut genesis_path = path_to_export.clone();
                genesis_path.push("genesis.json");
                std::fs::write(genesis_path, genesis_json).expect("can write genesis");

                let mut validator_state_path = path_to_export.clone();
                validator_state_path.push("priv_validator_state.json");
                let fresh_validator_state =
                    crate::testnet::generate::TestnetValidator::initial_state();
                std::fs::write(validator_state_path, fresh_validator_state)
                    .expect("can write validator state");
            }
            Upgrade::Testnet65 => { /* currently a no-op. */ }
        }
        Ok(())
    }
}
