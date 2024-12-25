//! Contains functions related to the migration script of Testnet74
#![allow(dead_code)]

use anyhow;
use cnidarium::{EscapedByteSlice, Snapshot, StateDelta, StateRead, StateWrite, Storage};
use futures::StreamExt as _;
use jmt::RootHash;
use penumbra_sdk_app::{app::StateReadExt as _, SUBSTORE_PREFIXES};
use penumbra_sdk_auction::{params::AuctionParameters, StateWriteExt};
use penumbra_sdk_dex::SwapExecution;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component as pb, StateReadProto, StateWriteProto};
use penumbra_sdk_sct::component::clock::{EpochManager, EpochRead};
use std::path::PathBuf;

use crate::network::generate::NetworkConfig;

/// Writes the auction parameters to the chain state.
async fn write_auction_parameters(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let params = AuctionParameters {};
    delta.put_auction_params(params);
    Ok(())
}

/// Updates arb execution output amounts to include the input amount instead
/// of reporting only profit (see #3790).
async fn fix_arb_execution_outputs(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let mut stream = delta.prefix_proto("dex/arb_execution/");
    while let Some(r) = stream.next().await {
        let (key, swap_ex_proto): (String, pb::dex::v1::SwapExecution) = r?;
        let mut swap_ex: SwapExecution = swap_ex_proto.try_into()?;
        swap_ex.output = swap_ex
            .input
            .asset_id
            .value(swap_ex.output.amount + swap_ex.input.amount);
        delta.put(key, swap_ex);
    }
    Ok(())
}

/// Update base liquidity index values to be proto-encoded. Previously they were stored as big-endian
/// encoded amounts, but in https://github.com/penumbra-zone/penumbra/pull/4188 they were changed
/// to be proto-encoded.
///
/// This will rewrite all values under the `dex/ab/` prefix to be proto-encoded.
async fn rewrite_base_liquidity_indices(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let prefix_key = "dex/ab/".as_bytes();
    tracing::trace!(prefix_key = ?EscapedByteSlice(&prefix_key), "updating base liquidity index values");
    let mut liquidity_stream = delta.nonverifiable_prefix_raw(&prefix_key).boxed();

    while let Some(r) = liquidity_stream.next().await {
        let (key, raw_amount): (Vec<u8>, Vec<u8>) = r?;
        tracing::info!(?key, raw_amount = ?EscapedByteSlice(&raw_amount), "migrating base liquidity index entry");

        let amount = Amount::from_be_bytes(raw_amount.as_slice().try_into()?);

        // Store the correctly formatted new value:
        delta.nonverifiable_put(key.clone(), amount);
        tracing::info!(
            key = ?EscapedByteSlice(&key),
            raw_amount = ?EscapedByteSlice(&raw_amount),
            ?amount,
            "updated base liquidity index"
        );
    }

    Ok(())
}

/// Update the ordering of liquidity position indices to return in descending order (see #4189)
///
/// Lookups for liquidity positions based on starting asset were ordered backwards
/// and returning the positions with the least liquidity first. This migration
/// needs to modify the keys stored under the nonverifiable `dex/ra/` prefix key to reverse
/// the ordering of the existing data.
async fn update_lp_index_order(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let prefix_key = "dex/ra/".as_bytes();
    tracing::trace!(prefix_key = ?EscapedByteSlice(&prefix_key), "updating liquidity position indices");
    let mut liquidity_stream = delta.nonverifiable_prefix_raw(&prefix_key).boxed();

    while let Some(r) = liquidity_stream.next().await {
        let (old_key, asset_id): (Vec<u8>, Vec<u8>) = r?;
        tracing::info!(?old_key, asset_id = ?EscapedByteSlice(&asset_id), "migrating asset liquidity");

        // Construct the new key:
        let mut new_key = [0u8; 55];
        new_key[0..7].copy_from_slice(b"dex/ra/");
        // The "from" asset ID remains the same in both keys.
        new_key[7..32 + 7].copy_from_slice(&old_key[7..32 + 7]);
        // Use the complement of the amount to ensure that the keys are ordered in descending order.
        let a_from_b = Amount::from_be_bytes(old_key[32 + 7..32 + 7 + 16].try_into()?);
        new_key[32 + 7..32 + 7 + 16].copy_from_slice(&(!a_from_b).to_be_bytes());

        // Delete the old incorrectly ordered key:
        delta.nonverifiable_delete(old_key.clone());

        // Store the correctly formatted new key:
        delta.nonverifiable_put_raw(new_key.to_vec(), asset_id);
        tracing::info!(
            new_key = ?EscapedByteSlice(&new_key),
            ?old_key,
            "updated liquidity index"
        );
    }

    Ok(())
}

/// Run the full migration, given an export path and a start time for genesis.
///
/// This migration script is responsible for:
///
/// - Updating the ordering of liquidity position indices to return in descending order (see #4189)
///     * nonverifiable: `dex/ra/`
/// - Updating arb execution output amounts to include the input amount instead of reporting only profit (see #3790)
///     * JMT: `dex/arb_execution/`
/// - Add `AuctionParameters` to the chain state
///     * JMT: `dex/auction_parameters`
/// - Update the base liquidity index values to be proto-encoded (see #4188)
///     * nonverifiable: `dex/ab/`
pub async fn migrate(
    storage: Storage,
    path_to_export: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    // Setup:
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
    let (migration_duration, post_upgrade_root_hash) = {
        let start_time = std::time::SystemTime::now();

        // Update LP index order.
        update_lp_index_order(&mut delta).await?;

        // Fix the arb execution output amounts.
        fix_arb_execution_outputs(&mut delta).await?;

        // Write auction parameters
        write_auction_parameters(&mut delta).await?;

        // Rewrite base liquidity indices as proto-encoded
        rewrite_base_liquidity_indices(&mut delta).await?;

        delta.put_block_height(0u64);
        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-upgrade root hash");

        (
            start_time.elapsed().expect("start time not set"),
            post_upgrade_root_hash,
        )
    };

    tracing::info!(?post_upgrade_root_hash, "post-upgrade root hash");
    storage.release().await;

    let rocksdb_dir = path_to_export.join("rocksdb");
    let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
    let migrated_state = storage.latest_snapshot();
    storage.release().await;

    // The migration is complete, now we need to generate a genesis file. To do this, we need
    // to lookup a validator view from the chain, and specify the post-upgrade app hash and
    // initial height.
    let chain_id = migrated_state.get_chain_id().await?;
    let app_state = penumbra_sdk_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = NetworkConfig::make_genesis(app_state.clone()).expect("can make genesis");
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
    let genesis = NetworkConfig::make_checkpoint(genesis, Some(checkpoint));

    let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
    tracing::info!("genesis: {}", genesis_json);
    let genesis_path = path_to_export.join("genesis.json");
    std::fs::write(genesis_path, genesis_json).expect("can write genesis");

    let validator_state_path = path_to_export.join("priv_validator_state.json");
    let fresh_validator_state = crate::network::generate::NetworkValidator::initial_state();
    std::fs::write(validator_state_path, fresh_validator_state).expect("can write validator state");

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
