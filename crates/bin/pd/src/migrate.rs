//! Logic for handling chain upgrades.
//!
//! When consensus-breaking changes are made to the Penumbra software,
//! node operators must coordinate to perform a chain upgrade.
//! This module declares how local `pd` state should be altered, if at all,
//! in order to be compatible with the network post-chain-upgrade.
mod testnet72;
mod testnet74;

use anyhow::{ensure, Context};
use futures::StreamExt as _;
use penumbra_governance::{StateReadExt, StateWriteExt};
use std::path::{Path, PathBuf};
use tracing::instrument;

use cnidarium::{StateDelta, StateRead, StateWrite, Storage};
use jmt::RootHash;
use penumbra_app::{app::StateReadExt as _, SUBSTORE_PREFIXES};
use penumbra_sct::component::clock::{EpochManager, EpochRead};

use crate::testnet::generate::TestnetConfig;

use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;

/// The kind of migration that should be performed.
#[derive(Debug)]
pub enum Migration {
    /// Set the chain's halt bit to `false`.
    ReadyToStart,
    /// A simple migration: adds a key to the consensus state.
    /// This is useful for testing upgrade mechanisms, including in production.
    SimpleMigration,
    /// Testnet-70 migration: move swap executions from the jmt to nv-storage.
    Testnet70,
    /// Testnet-72 migration:
    /// - Migrate `BatchSwapOutputData` to new protobuf, replacing epoch height with index.
    Testnet72,
    /// Testnet-74 migration:
    /// - Update the base liquidity index to order routable pairs by descending liquidity
    /// - Update arb executions to include the amount of filled input in the output
    /// - Add `AuctionParameters` to the consensus state
    Testnet74,
}

impl Migration {
    #[instrument(skip(path_to_export, genesis_start, force))]
    pub async fn migrate(
        &self,
        path_to_export: PathBuf,
        genesis_start: Option<tendermint::time::Time>,
        force: bool,
    ) -> anyhow::Result<()> {
        tracing::debug!(
            ?path_to_export,
            ?genesis_start,
            ?force,
            "preparing to run migration!"
        );
        let rocksdb_dir = path_to_export.join("rocksdb");
        let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
        let export_state = storage.latest_snapshot();
        let root_hash = export_state.root_hash().await.expect("can get root hash");
        let app_hash_pre_migration: RootHash = root_hash.into();
        let height = export_state
            .get_block_height()
            .await
            .expect("can get block height");

        tracing::debug!(?app_hash_pre_migration, current_height = ?height, ?force,
            "determining if the chain is halted and the migration is allowed to run");

        ensure!(
            export_state.is_chain_halted().await || force,
            "to run a migration, the chain halt bit must be set to `true` or use the `--force` cli flag"
        );

        tracing::info!(?app_hash_pre_migration, pre_upgrade_height = ?height, ?self, "started migration");
        match self {
            Migration::ReadyToStart => {
                let mut delta = StateDelta::new(export_state);
                delta.signal_halt();
                let _ = storage.commit_in_place(delta).await?;
                tracing::info!(
                    "migration completed: halt bit is turned off, chain is ready to start"
                );
                Ok(())
            }
            Migration::SimpleMigration => {
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
                let app_state = penumbra_app::genesis::Content {
                    chain_id,
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
                let genesis_path = path_to_export.join("genesis.json");
                std::fs::write(genesis_path, genesis_json).expect("can write genesis");

                let validator_state_path = path_to_export.join("priv_validator_state.json");
                let fresh_validator_state =
                    crate::testnet::generate::TestnetValidator::initial_state();
                std::fs::write(validator_state_path, fresh_validator_state)
                    .expect("can write validator state");
                Ok(())
            }
            Migration::Testnet70 => {
                // Our goal is to fetch all swap executions from the jmt and store them in nv-storage.
                // In particular, we want to make sure that client lookups for (height, trading pair)
                // resolve to the same value as before.

                // Setup:
                let start_time = std::time::SystemTime::now();
                let rocksdb_dir = path_to_export.join("rocksdb");
                let storage =
                    Storage::load(rocksdb_dir.clone(), SUBSTORE_PREFIXES.to_vec()).await?;
                let export_state = storage.latest_snapshot();
                let root_hash = export_state.root_hash().await.expect("can get root hash");
                let pre_upgrade_root_hash: RootHash = root_hash.into();
                let pre_upgrade_height = export_state
                    .get_block_height()
                    .await
                    .expect("can get block height");
                let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

                // We initialize a `StateDelta` and start by reaching into the JMT for all entries matching the
                // swap execution prefix. Then, we write each entry to the nv-storage.
                let mut delta = StateDelta::new(export_state);

                let prefix_key = "dex/swap_execution/";
                let mut swap_execution_stream = delta.prefix_raw(prefix_key);

                while let Some(r) = swap_execution_stream.next().await {
                    let (key, swap_execution) = r?;
                    tracing::info!("migrating swap execution: {}", key);
                    delta.nonverifiable_put_raw(key.into_bytes(), swap_execution);
                }

                delta.put_block_height(0u64);

                let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
                tracing::info!(?post_upgrade_root_hash, "post-upgrade root hash");

                let migration_duration = start_time.elapsed().expect("start time not set");

                // Reload storage so we can make reads against its migrated state:
                storage.release().await;
                let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
                let migrated_state = storage.latest_snapshot();
                storage.release().await;

                // The migration is complete, now we need to generate a genesis file. To do this, we need
                // to lookup a validator view from the chain, and specify the post-upgrade app hash and
                // initial height.
                let chain_id = migrated_state.get_chain_id().await?;
                let app_state = penumbra_app::genesis::Content {
                    chain_id,
                    ..Default::default()
                };
                let mut genesis =
                    TestnetConfig::make_genesis(app_state.clone()).expect("can make genesis");
                genesis.app_hash = post_upgrade_root_hash
                    .0
                    .to_vec()
                    .try_into()
                    .expect("infaillible conversion");
                genesis.initial_height = post_upgrade_height as i64;
                genesis.genesis_time = genesis_start.unwrap_or_else(|| {
                    let now = tendermint::time::Time::now();
                    tracing::info!(%now, "no genesis time provided, detecting a testing setup");
                    now
                });
                let checkpoint = post_upgrade_root_hash.0.to_vec();
                let genesis = TestnetConfig::make_checkpoint(genesis, Some(checkpoint));

                let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
                tracing::info!("genesis: {}", genesis_json);
                let genesis_path = path_to_export.join("genesis.json");
                std::fs::write(genesis_path, genesis_json).expect("can write genesis");

                let validator_state_path = path_to_export.join("priv_validator_state.json");
                let fresh_validator_state =
                    crate::testnet::generate::TestnetValidator::initial_state();
                std::fs::write(validator_state_path, fresh_validator_state)
                    .expect("can write validator state");

                tracing::info!(
                    pre_upgrade_height,
                    post_upgrade_height,
                    ?pre_upgrade_root_hash,
                    ?post_upgrade_root_hash,
                    duration = migration_duration.as_secs(),
                    "successful migration!"
                );

                Ok(())
            }
            Migration::Testnet72 => testnet72::migrate(path_to_export, genesis_start).await,
            Migration::Testnet74 => testnet74::migrate(path_to_export, genesis_start).await,
        }
    }
}

/// Compress single directory to gzipped tar archive. Accepts an Option for naming
/// the subdir within the tar archive, which defaults to ".", meaning no nesting.
pub fn archive_directory(
    src_directory: PathBuf,
    archive_filepath: PathBuf,
    subdir_within_archive: Option<String>,
) -> anyhow::Result<()> {
    // Don't clobber an existing target archive.
    if archive_filepath.exists() {
        tracing::error!(
            "export archive filepath already exists: {}",
            archive_filepath.display()
        );
        anyhow::bail!("refusing to overwrite existing archive");
    }

    tracing::info!(
        "creating archive {} -> {}",
        src_directory.display(),
        archive_filepath.display()
    );
    let tarball_file = File::create(&archive_filepath)
        .context("failed to create file for archive: check parent directory and permissions")?;
    let enc = GzEncoder::new(tarball_file, Compression::default());
    let mut tarball = tar::Builder::new(enc);
    let subdir_within_archive = subdir_within_archive.unwrap_or(String::from("."));
    tarball
        .append_dir_all(subdir_within_archive, src_directory.as_path())
        .context("failed to package archive contents")?;
    Ok(())
}

/// Read the last block timestamp from the pd state.
pub async fn last_block_timestamp(home: PathBuf) -> anyhow::Result<tendermint::Time> {
    let rocksdb = home.join("rocksdb");
    let storage = Storage::load(rocksdb, SUBSTORE_PREFIXES.to_vec())
        .await
        .context("error loading store for timestamp")?;
    let state = storage.latest_snapshot();
    let last_block_time = state
        .get_block_timestamp()
        .await
        .context("error reading latest block timestamp")?;
    storage.release().await;
    Ok(last_block_time)
}

#[instrument(skip_all)]
pub async fn migrate_comet_data(
    comet_home: PathBuf,
    new_genesis_file: PathBuf,
) -> anyhow::Result<()> {
    tracing::info!(?comet_home, ?new_genesis_file, "migrating comet data");

    // Read the contents of new_genesis_file into a serde_json::Value and pull out .initial_height
    let genesis_contents =
        std::fs::read_to_string(new_genesis_file).context("error reading new genesis file")?;
    let genesis_json: serde_json::Value =
        serde_json::from_str(&genesis_contents).context("error parsing new genesis file")?;
    tracing::info!(?genesis_json, "parsed genesis file");
    let initial_height = genesis_json["initial_height"]
        .as_str()
        .context("error reading initial_height from genesis file")?
        .parse::<u64>()?;

    // Write the genesis data to HOME/config/genesis.json
    let genesis_file = comet_home.join("config").join("genesis.json");
    tracing::info!(?genesis_file, "writing genesis file to comet config");
    std::fs::write(genesis_file, genesis_contents)
        .context("error writing genesis file to comet config")?;

    // Adjust the high-water mark in priv_validator_state.json but don't decrease it
    adjust_priv_validator_state(&comet_home, initial_height)?;

    // Delete other cometbft data.
    clear_comet_data(&comet_home)?;

    Ok(())
}

#[instrument(skip_all)]
fn adjust_priv_validator_state(comet_home: &Path, initial_height: u64) -> anyhow::Result<()> {
    let priv_validator_state = comet_home.join("data").join("priv_validator_state.json");
    let current_state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&priv_validator_state)?)?;

    let current_height = current_state["height"]
        .as_str()
        .context("error reading height from priv_validator_state.json")?
        .parse::<u64>()?;
    if current_height < initial_height {
        tracing::info!(
            "increasing height in priv_validator_state from {} to {}",
            current_height,
            initial_height
        );
        let new_state = serde_json::json!({
            "height": initial_height.to_string(), // Important to use to_string here as if protojson
            "round": 0,
            "step": 0,
        });
        tracing::info!(?new_state, "updated priv_validator_state.json");
        std::fs::write(
            &priv_validator_state,
            &serde_json::to_string_pretty(&new_state)?,
        )?;
    } else {
        anyhow::bail!(
            "priv_validator_state height {} is already greater than or equal to initial_height {}",
            current_height,
            initial_height
        );
    }

    Ok(())
}

#[instrument(skip_all)]
fn clear_comet_data(comet_home: &Path) -> anyhow::Result<()> {
    let data_dir = comet_home.join("data");

    /*
    N.B. We want to preserve the `tx_index.db` directory.
    Doing so will allow CometBFT to reference historical transactions behind the upgrade boundary.
     */
    for subdir in &["evidence.db", "state.db", "blockstore.db", "cs.wal"] {
        let path = data_dir.join(subdir);
        if path.exists() {
            tracing::info!(?path, "removing file");
            std::fs::remove_dir_all(path)?;
        }
    }

    Ok(())
}
