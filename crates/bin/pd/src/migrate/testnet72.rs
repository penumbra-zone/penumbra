//! Contains functions related to the migration script of Testnet72
#![allow(dead_code)]
use anyhow;
use cnidarium::{Snapshot, StateDelta, StateRead, StateWrite, Storage};
use futures::StreamExt as _;
use jmt::RootHash;
use penumbra_sdk_app::app::StateReadExt as _;
use penumbra_sdk_app::SUBSTORE_PREFIXES;
use penumbra_sdk_proto::core::component::sct::v1::query_service_server::QueryService;
use penumbra_sdk_proto::penumbra::core::component as pb;
use penumbra_sdk_proto::StateWriteProto;
use penumbra_sdk_sct::component::clock::{EpochManager, EpochRead};
use penumbra_sdk_sct::component::rpc::Server as SctServer;
use penumbra_sdk_tct::Position;
use prost::Message;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::IntoRequest;

use crate::network::generate::NetworkConfig;

/// The context holding various query services we need to help perform the migration.
#[derive(Clone)]
struct Context {
    sct_server: Arc<SctServer>,
}

impl Context {
    /// Create a new context from the state storage.
    fn new(storage: Storage) -> Self {
        Self {
            sct_server: Arc::new(SctServer::new(storage)),
        }
    }

    /// Use storage to lookup the index of an epoch based on its starting heights
    async fn epoch_height_to_index(&self, epoch_starting_height: u64) -> anyhow::Result<u64> {
        Ok(self
            .sct_server
            .epoch_by_height(
                pb::sct::v1::EpochByHeightRequest {
                    height: epoch_starting_height,
                }
                .into_request(),
            )
            .await?
            .into_inner()
            .epoch
            .expect(&format!(
                "epoch at height {} should be present",
                epoch_starting_height
            ))
            .index)
    }

    /// Translate the protobuf for a BSOD by populating the correct data and emptying the
    /// deprecated field.
    #[allow(deprecated)]
    async fn translate_bsod(
        &self,
        bsod: pb::dex::v1::BatchSwapOutputData,
    ) -> anyhow::Result<pb::dex::v1::BatchSwapOutputData> {
        let sct_position_prefix: u64 = {
            let epoch = self
                .epoch_height_to_index(bsod.epoch_starting_height)
                .await?;
            Position::from((
                u16::try_from(epoch).expect("epoch should fit in 16 bits"),
                u16::try_from(bsod.height - bsod.epoch_starting_height)
                    .expect("block index should fit in 16 bits"),
                0,
            ))
            .into()
        };
        Ok(pb::dex::v1::BatchSwapOutputData {
            sct_position_prefix,
            epoch_starting_height: Default::default(),
            ..bsod
        })
    }

    async fn translate_compact_block(
        &self,
        compact_block: pb::compact_block::v1::CompactBlock,
    ) -> anyhow::Result<pb::compact_block::v1::CompactBlock> {
        let mut swap_outputs = Vec::with_capacity(compact_block.swap_outputs.len());
        for bsod in compact_block.swap_outputs {
            swap_outputs.push(self.translate_bsod(bsod).await?);
        }
        Ok(pb::compact_block::v1::CompactBlock {
            swap_outputs,
            ..compact_block
        })
    }
}

/// Translate all of the BSODs inside dex storage to the new format.
async fn translate_dex_storage(
    ctx: Context,
    delta: &mut StateDelta<Snapshot>,
) -> anyhow::Result<()> {
    let mut stream = delta.prefix_raw("dex/output/");
    while let Some(r) = stream.next().await {
        let (key, bsod_bytes) = r?;
        let bsod = pb::dex::v1::BatchSwapOutputData::decode(bsod_bytes.as_slice())?;
        let bsod = ctx.translate_bsod(bsod).await?;
        delta.put_proto(key, bsod);
    }
    Ok(())
}

/// Translate all of the compact block storage to hold the new BSOD data inside the compact blocks.
async fn translate_compact_block_storage(
    ctx: Context,
    delta: &mut StateDelta<Snapshot>,
) -> anyhow::Result<()> {
    let mut stream = delta.nonverifiable_prefix_raw("compactblock/".as_bytes());
    while let Some(r) = stream.next().await {
        let (key, compactblock_bytes) = r?;
        let block = pb::compact_block::v1::CompactBlock::decode(compactblock_bytes.as_slice())?;
        let block = ctx.translate_compact_block(block).await?;
        delta.nonverifiable_put_raw(key, block.encode_to_vec());
    }
    Ok(())
}

/// Run the full migration, given an export path and a start time for genesis.
pub async fn migrate(
    storage: Storage,
    path_to_export: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    let export_state = storage.latest_snapshot();
    let root_hash = export_state.root_hash().await.expect("can get root hash");
    let pre_upgrade_root_hash: RootHash = root_hash.into();
    let pre_upgrade_height = export_state
        .get_block_height()
        .await
        .expect("can get block height");
    let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

    let mut delta = StateDelta::new(export_state);
    let (migration_duration, post_upgrade_root_hash) = {
        let start_time = std::time::SystemTime::now();
        let ctx = Context::new(storage.clone());

        // Translate inside dex storage.
        translate_dex_storage(ctx.clone(), &mut delta).await?;
        // Translate inside compact block storage.
        translate_compact_block_storage(ctx.clone(), &mut delta).await?;

        delta.put_block_height(0u64);
        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-upgrade root hash");

        (
            start_time.elapsed().expect("start time not set"),
            post_upgrade_root_hash,
        )
    };

    storage.release().await;

    let rocksdb_dir = path_to_export.join("rocksdb");
    let storage = Storage::load(rocksdb_dir, SUBSTORE_PREFIXES.to_vec()).await?;
    let migrated_state = storage.latest_snapshot();

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
