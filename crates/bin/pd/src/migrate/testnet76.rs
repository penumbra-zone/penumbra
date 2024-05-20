//! Contains functions related to the migration script of Testnet74

use anyhow;
use cnidarium::{Snapshot, StateDelta, Storage};
use futures::TryStreamExt;
use jmt::RootHash;
use pbjson_types::Any;
use penumbra_app::{app::StateReadExt as _, SUBSTORE_PREFIXES};
use penumbra_asset::Balance;
use penumbra_auction::auction::dutch::DutchAuction;
use penumbra_proto::{DomainType, StateReadProto, StateWriteProto};
use penumbra_sct::component::clock::{EpochManager, EpochRead};
use std::path::PathBuf;
use tracing::instrument;

use crate::testnet::generate::TestnetConfig;

#[instrument(skip_all)]
/// Reconstruct a correct tally of the auction component's VCB balance.
/// This is achieved by:
/// 1. Iterating over all auctions in the chain state.
/// 2. Summing the input and output reserves of each auction.
///    NB: This is sufficient because auctions with deployed LPs have value that is
/// //     *outside* of the auction component, and recorded in the DEX VCB instead.
/// 3. Writing the aggregate VCB balance for each asset to the chain state.
async fn heal_auction_vcb(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let key_prefix_auctions = penumbra_auction::state_key::auction_store::prefix();
    let all_auctions = delta
        .prefix_proto::<Any>(&key_prefix_auctions)
        .map_ok(|(_, v)| DutchAuction::decode(v.value).expect("only dutch auctions"))
        .try_collect::<Vec<DutchAuction>>()
        .await?;

    let total_vcb = all_auctions
        .into_iter()
        .filter(|auction| auction.state.sequence <= 1)
        .fold(Balance::zero(), |acc, auction| {
            let input_reserves = penumbra_asset::Value {
                asset_id: auction.description.input.asset_id,
                amount: auction.state.input_reserves,
            };

            let output_reserves = penumbra_asset::Value {
                asset_id: auction.description.output_id,
                amount: auction.state.output_reserves,
            };

            tracing::debug!(id = ?auction.description.id(), ?input_reserves, ?output_reserves, "aggregating auction into the component's VCB balance");

            acc + Balance::from(input_reserves) + Balance::from(output_reserves)
        });

    for value in total_vcb.provided() {
        tracing::debug!(?value, "writing aggregate VCB balance for asset");
        let key_vcb_balance =
            penumbra_auction::state_key::value_balance::for_asset(&value.asset_id);
        delta.put(key_vcb_balance, value.amount);
    }

    Ok(())
}

/// Run the full migration, given an export path and a start time for genesis.
///
/// Menu:
/// - Reconstruct a correct VCB balance for the auction component.
#[instrument(skip_all)]
pub async fn migrate(
    path_to_export: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    // Setup:
    let rocksdb_dir = path_to_export.join("rocksdb");
    let storage = Storage::load(rocksdb_dir.clone(), SUBSTORE_PREFIXES.to_vec()).await?;
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

        // Reconstruct a VCB balance for the auction component.
        heal_auction_vcb(&mut delta).await?;

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
    let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
    let migrated_state = storage.latest_snapshot();

    // The migration is complete, now we need to generate a genesis file. To do this, we need
    // to lookup a validator view from the chain, and specify the post-upgrade app hash and
    // initial height.
    let chain_id = migrated_state.get_chain_id().await?;
    let app_state = penumbra_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = TestnetConfig::make_genesis(app_state.clone()).expect("can make genesis");
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
    let fresh_validator_state = crate::testnet::generate::TestnetValidator::initial_state();
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
