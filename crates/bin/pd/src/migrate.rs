//! Logic for handling chain upgrades.
//!
//! When consensus-breaking changes are made to the Penumbra software,
//! node operators must coordinate to perform a chain upgrade.
//! This module declares how local `pd` state should be altered, if at all,
//! in order to be compatible with the network post-chain-upgrade.
mod mainnet1;
mod mainnet2;
mod reset_halt_bit;
mod simple;
mod testnet72;
mod testnet74;
mod testnet76;
mod testnet77;
mod testnet78;

use anyhow::{ensure, Context};
use penumbra_governance::StateReadExt;
use penumbra_sct::component::clock::EpochRead;
use std::path::{Path, PathBuf};
use tracing::instrument;

use cnidarium::Storage;
use penumbra_app::SUBSTORE_PREFIXES;

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
    /// Testnet-72 migration:
    /// - Migrate `BatchSwapOutputData` to new protobuf, replacing epoch height with index.
    Testnet72,
    /// Testnet-74 migration:
    /// - Update the base liquidity index to order routable pairs by descending liquidity
    /// - Update arb executions to include the amount of filled input in the output
    /// - Add `AuctionParameters` to the consensus state
    Testnet74,
    /// Testnet-76 migration:
    /// - Heal the auction component's VCB tally.
    /// - Update FMD parameters to new protobuf structure.
    Testnet76,
    /// Testnet-77 migration:
    /// - Reset the halt bit
    Testnet77,
    /// Testnet-78 migration:
    /// - Truncate various user-supplied `String` fields to a maximum length.
    /// - Populate the DEX NV price idnexes with position data
    Testnet78,
    /// Mainnet-1 migration:
    /// - Restore IBC packet commitments for improperly handled withdrawal attempts
    Mainnet1,
    /// Mainnet-2 migration:
    /// - no-op
    Mainnet2,
}

impl Migration {
    #[instrument(skip(pd_home, genesis_start, force))]
    pub async fn migrate(
        &self,
        pd_home: PathBuf,
        comet_home: Option<PathBuf>,
        genesis_start: Option<tendermint::time::Time>,
        force: bool,
    ) -> anyhow::Result<()> {
        tracing::debug!(
            ?pd_home,
            ?genesis_start,
            ?force,
            "preparing to run migration!"
        );
        let rocksdb_dir = pd_home.join("rocksdb");
        let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
        ensure!(
            storage.latest_snapshot().is_chain_halted().await || force,
            "to run a migration, the chain halt bit must be set to `true` or use the `--force` cli flag"
        );
        tracing::info!("started migration");

        // If this is `ReadyToStart`, we need to reset the halt bit and return early.
        if let Migration::ReadyToStart = self {
            reset_halt_bit::migrate(storage, pd_home, genesis_start).await?;
            return Ok(());
        }

        match self {
            Migration::SimpleMigration => {
                simple::migrate(storage, pd_home.clone(), genesis_start).await?
            }
            Migration::Mainnet1 => {
                mainnet1::migrate(storage, pd_home.clone(), genesis_start).await?;
            }
            Migration::Mainnet2 => {
                mainnet2::migrate(storage, pd_home.clone(), genesis_start).await?;
            }
            // We keep historical migrations around for now, this will help inform an abstracted
            // design. Feel free to remove it if it's causing you trouble.
            _ => unimplemented!("the specified migration is unimplemented"),
        }

        if let Some(comet_home) = comet_home {
            let genesis_path = pd_home.join("genesis.json");
            migrate_comet_data(comet_home, genesis_path).await?;
        }

        Ok(())
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
        .get_current_block_timestamp()
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
