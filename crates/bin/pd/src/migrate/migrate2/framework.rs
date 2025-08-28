use anyhow::Result;
use cnidarium::{StateDelta, Storage};
use jmt::RootHash;
use penumbra_sdk_app::app::StateReadExt as _;
use penumbra_sdk_app::app_version::migrate_app_version;
use penumbra_sdk_app::SUBSTORE_PREFIXES;
use penumbra_sdk_governance::StateWriteExt;
use penumbra_sdk_sct::component::clock::{EpochManager, EpochRead};
use std::path::PathBuf;
use tracing::instrument;

use crate::network::generate::NetworkConfig;

pub trait Migration {
    fn name(&self) -> &'static str;

    /// If the migration results in a bumped app version.
    fn target_app_version(&self) -> Option<u64>;

    #[instrument(skip(self, pd_home, _comet_home))]
    async fn prepare(
        &self,
        pd_home: &PathBuf,
        _comet_home: Option<&PathBuf>,
    ) -> Result<(RootHash, u64)> {
        let rocksdb_dir = pd_home.join("rocksdb");
        let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
        let initial_state = storage.latest_snapshot();

        let root_hash = initial_state
            .root_hash()
            .await
            .expect("chain state has a root hash");
        let pre_upgrade_root_hash: RootHash = root_hash.into();
        let pre_upgrade_height = initial_state
            .get_block_height()
            .await
            .expect("chain state has a block height");

        storage.release().await;

        Ok((pre_upgrade_root_hash, pre_upgrade_height))
    }

    #[instrument(skip(self, pd_home, _comet_home))]
    async fn migrate(
        &self,
        pd_home: &PathBuf,
        _comet_home: Option<&PathBuf>,
    ) -> Result<(RootHash, u64)> {
        let rocksdb_dir = pd_home.join("rocksdb");
        let storage = Storage::load(rocksdb_dir.clone(), SUBSTORE_PREFIXES.to_vec()).await?;
        let initial_state = storage.latest_snapshot();

        let _chain_id = initial_state.get_chain_id().await?;
        let root_hash = initial_state
            .root_hash()
            .await
            .expect("chain state has a root hash");
        let _pre_upgrade_root_hash: RootHash = root_hash.into();
        let pre_upgrade_height = initial_state
            .get_block_height()
            .await
            .expect("chain state has a block height");
        let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

        let mut delta = StateDelta::new(initial_state);

        if let Some(target_version) = self.target_app_version() {
            migrate_app_version(&mut delta, target_version).await?;
        }

        self.migrate_inner(&mut delta).await?;

        delta.ready_to_start();
        delta.put_block_height(0u64);

        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-migration root hash");

        storage.release().await;

        Ok((post_upgrade_root_hash, post_upgrade_height))
    }

    async fn migrate_inner(&self, _delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        Ok(())
    }

    #[instrument(skip(self, pd_home, comet_home, post_upgrade_root_hash))]
    async fn complete(
        &self,
        pd_home: &PathBuf,
        comet_home: Option<&PathBuf>,
        post_upgrade_root_hash: RootHash,
        post_upgrade_height: u64,
        genesis_start: Option<tendermint::time::Time>,
    ) -> Result<()> {
        let rocksdb_dir = pd_home.join("rocksdb");
        let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
        let state = storage.latest_snapshot();
        let chain_id = state.get_chain_id().await?;
        storage.release().await;

        let app_state = penumbra_sdk_app::genesis::Content {
            chain_id,
            ..Default::default()
        };
        let mut genesis = NetworkConfig::make_genesis(app_state.clone()).expect("can make genesis");
        genesis.app_hash = post_upgrade_root_hash
            .0
            .to_vec()
            .try_into()
            .expect("infallible conversion");

        genesis.initial_height = post_upgrade_height as i64;
        genesis.genesis_time = genesis_start.unwrap_or_else(|| {
            let now = tendermint::time::Time::now();
            tracing::info!(%now, "no genesis time provided, detecting a testing setup");
            now
        });
        let checkpoint = post_upgrade_root_hash.0.to_vec();
        let genesis = NetworkConfig::make_checkpoint(genesis, Some(checkpoint));
        let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
        tracing::info!("genesis: {}", genesis_json);
        let genesis_path = pd_home.join("genesis.json");
        std::fs::write(genesis_path, genesis_json).expect("can write genesis");

        let validator_state_path = pd_home.join("priv_validator_state.json");
        let fresh_validator_state = crate::network::generate::NetworkValidator::initial_state();
        std::fs::write(validator_state_path, fresh_validator_state)
            .expect("can write validator state");

        if let Some(comet_home) = comet_home {
            let genesis_path = pd_home.join("genesis.json");
            crate::migrate::migrate_comet_data(comet_home.to_path_buf(), genesis_path).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, pd_home, comet_home))]
    async fn run(
        &self,
        pd_home: PathBuf,
        comet_home: Option<PathBuf>,
        genesis_start: Option<tendermint::time::Time>,
    ) -> Result<()> {
        let (pre_upgrade_root_hash, pre_upgrade_height) =
            self.prepare(&pd_home, comet_home.as_ref()).await?;
        tracing::info!(
            ?pre_upgrade_root_hash,
            pre_upgrade_height,
            migration = self.name(),
            "starting migration"
        );

        let start_time = std::time::SystemTime::now();
        let (post_upgrade_root_hash, post_upgrade_height) =
            self.migrate(&pd_home, comet_home.as_ref()).await?;
        let migration_duration = start_time.elapsed().expect("start time is set");

        self.complete(
            &pd_home,
            comet_home.as_ref(),
            post_upgrade_root_hash,
            post_upgrade_height,
            genesis_start,
        )
        .await?;

        tracing::info!(
            pre_upgrade_height,
            post_upgrade_height,
            ?pre_upgrade_root_hash,
            ?post_upgrade_root_hash,
            duration = migration_duration.as_secs(),
            migration = self.name(),
            "successful migration!"
        );

        Ok(())
    }
}
