use crate::Component;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_chain::{genesis, sync::CompactBlock, Epoch, NoteSource, StateReadExt as _};
use penumbra_crypto::{asset, note, Nullifier, Value};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateTransaction, StateWrite};
use penumbra_tct as tct;
use tct::Tree;
use tendermint::abci;

use crate::shielded_pool::state_key;

use super::{NoteManager, SupplyWrite};

pub struct ShieldedPool {}

#[async_trait]
impl Component for ShieldedPool {
    // #[instrument(name = "shielded_pool", skip(state, app_state))]
    async fn init_chain(state: &mut StateTransaction, app_state: &genesis::AppState) {
        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            assert_ne!(
                allocation.amount, 0u64,
                "Genesis allocations contain empty note",
            );

            let unit = asset::REGISTRY.parse_unit(&allocation.denom);

            state.register_denom(&unit.base()).await.unwrap();
            state
                .mint_note(
                    Value {
                        amount: (allocation.amount * 10u64.pow(unit.exponent().into())).into(),
                        asset_id: unit.id(),
                    },
                    &allocation.address,
                    NoteSource::Genesis,
                )
                .await
                .unwrap();
        }

        let mut compact_block = state.stub_compact_block();
        let mut note_commitment_tree = state.stub_note_commitment_tree().await;

        // Hard-coded to zero because we are in the genesis block
        // Tendermint starts blocks at 1, so this is a "phantom" compact block
        compact_block.height = 0;

        // Add current FMD parameters to the initial block.
        compact_block.fmd_parameters = Some(state.get_current_fmd_parameters().await.unwrap());

        // Close the genesis block
        state
            .finish_nct_block(&mut compact_block, &mut note_commitment_tree)
            .await;

        state
            .write_compactblock_and_nct(compact_block, note_commitment_tree)
            .await
            .expect("unable to write compactblock and nct");
    }

    // #[instrument(name = "shielded_pool", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    // #[instrument(name = "shielded_pool", skip(state, _end_block))]
    async fn end_block(state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {
        // Get the current block height
        let height = state.height().await;

        // Set the height of the compact block and save it.
        let mut compact_block = state.stub_compact_block();
        compact_block.height = height;
        state.stub_put_compact_block(compact_block);

        // TODO: execute any scheduled DAO spend transactions for this block

        // We need to reload the compact block here, in case it was
        // edited during the preceding method calls.
        let mut compact_block = state.stub_compact_block();
        // Close the block in the NCT
        let mut note_commitment_tree = state.stub_note_commitment_tree().await;
        state
            .finish_nct_block(&mut compact_block, &mut note_commitment_tree)
            .await;

        state
            .write_compactblock_and_nct(compact_block, note_commitment_tree)
            .await
            .expect("unable to write compactblock and nct");
    }
}

// TODO: split into different extension traits
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn stub_note_commitment_tree(&self) -> tct::Tree {
        match self
            .nonconsensus_get_raw(state_key::internal::stub_note_commitment_tree().as_bytes())
            .await
            .unwrap()
        {
            Some(bytes) => bincode::deserialize(&bytes).unwrap(),
            None => tct::Tree::new(),
        }
    }

    fn stub_compact_block(&self) -> CompactBlock {
        self.object_get(state_key::internal::stub_compact_block())
            .cloned()
            .unwrap_or_default()
    }

    async fn note_source(&self, note_commitment: note::Commitment) -> Result<Option<NoteSource>> {
        self.get(&state_key::note_source(&note_commitment)).await
    }

    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        self.get(&state_key::compact_block(height)).await
    }

    // #[instrument(skip(self))]
    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(source) = self
            .get::<NoteSource>(&state_key::spent_nullifier_lookup(nullifier))
            .await?
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                source,
            ));
        }
        Ok(())
    }

    /// Returns the NCT anchor for the given height.
    async fn anchor_by_height(&self, height: u64) -> Result<Option<tct::Root>> {
        self.get(&state_key::anchor_by_height(height)).await
    }

    /// Checks whether a claimed NCT anchor is a previous valid state root.
    async fn check_claimed_anchor(&self, anchor: tct::Root) -> Result<()> {
        if let Some(anchor_height) = self
            .get_proto::<u64>(&state_key::anchor_lookup(anchor))
            .await?
        {
            tracing::debug!(?anchor, ?anchor_height, "anchor is valid");
            Ok(())
        } else {
            Err(anyhow!(
                "provided anchor {} is not a valid NCT root",
                anchor
            ))
        }
    }

    /// Finish the block in the NCT.
    ///
    /// TODO: where should this live
    /// re-evaluate
    // #[instrument(skip(self))]
    async fn finish_nct_block(
        &self,
        compact_block: &mut CompactBlock,
        note_commitment_tree: &mut Tree,
    ) {
        // Get the current block height
        let height = compact_block.height;

        // Close the block in the TCT
        let block_root = note_commitment_tree
            .end_block()
            .expect("ending a block in the note commitment tree can never fail");

        // Put the block root in the compact block
        compact_block.block_root = block_root;

        // If the block ends an epoch, also close the epoch in the TCT
        if Epoch::from_height(
            height,
            self.get_chain_params()
                .await
                .expect("chain params request must succeed")
                .epoch_duration,
        )
        .is_epoch_end(height)
        {
            tracing::debug!(?height, "end of epoch");

            // TODO: Put updated FMD parameters in the compact block

            let epoch_root = note_commitment_tree
                .end_epoch()
                .expect("ending an epoch in the note commitment tree can never fail");

            // Put the epoch root in the compact block
            compact_block.epoch_root = Some(epoch_root);
        }
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    // TODO: remove this entirely post-integration. This is slow but intended as
    // a drop-in replacement so we can avoid really major code changes.
    //
    // Instead, replace with a mechanism that builds up queued note payloads
    // and builds the compact block only at the end of the block, so we don't need
    // the NCT at all until end_block, and then serialization round trip doesn't matter.
    fn stub_put_note_commitment_tree(&mut self, tree: &tct::Tree) {
        let bytes = bincode::serialize(&tree).unwrap();
        self.nonconsensus_put_raw(
            state_key::internal::stub_note_commitment_tree()
                .as_bytes()
                .to_vec(),
            bytes,
        );
    }

    // TODO: remove this entirely post-integration. This is slow but intended as
    // a drop-in replacement so we can avoid really major code changes.
    //
    // Instead, replace with a mechanism that builds up the parts and builds the
    // compact block only at the end of the block, so we don't need the NCT at
    // all until end_block, and then serialization round trip doesn't matter.
    fn stub_put_compact_block(&mut self, compact_block: CompactBlock) {
        self.object_put(state_key::internal::stub_compact_block(), compact_block);
    }

    /// Writes a completed compact block into the public state.
    fn set_compact_block(&mut self, compact_block: CompactBlock) {
        let height = compact_block.height;
        self.put(state_key::compact_block(height), compact_block);
    }

    fn set_nct_anchor(&mut self, height: u64, nct_anchor: tct::Root) {
        tracing::debug!(?height, ?nct_anchor, "writing anchor");

        // Write the NCT anchor both as a value, so we can look it up,
        self.put(state_key::anchor_by_height(height), nct_anchor);
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::anchor_lookup(nct_anchor),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height,
        );
    }

    fn set_nct_block_anchor(&mut self, height: u64, nct_block_anchor: tct::builder::block::Root) {
        tracing::debug!(?height, ?nct_block_anchor, "writing block anchor");

        // Write the NCT block anchor both as a value, so we can look it up,
        self.put(state_key::block_anchor_by_height(height), nct_block_anchor);
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::block_anchor_lookup(nct_block_anchor),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height,
        );
    }

    fn set_nct_epoch_anchor(&mut self, index: u64, nct_block_anchor: tct::builder::epoch::Root) {
        tracing::debug!(?index, ?nct_block_anchor, "writing epoch anchor");

        // Write the NCT epoch anchor both as a value, so we can look it up,
        self.put(state_key::epoch_anchor_by_index(index), nct_block_anchor);
        // and as a key, so we can query for it.
        self.put_proto(
            state_key::epoch_anchor_lookup(nct_block_anchor),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            index,
        );
    }

    // #[instrument(skip(self))]
    async fn write_compactblock_and_nct(
        &mut self,
        compact_block: CompactBlock,
        nct: tct::Tree,
    ) -> Result<()> {
        let height = compact_block.height;

        // Write the note commitment tree anchor:
        self.set_nct_anchor(height, nct.root());
        // Write the current block anchor:
        self.set_nct_block_anchor(height, compact_block.block_root);
        // Write the current epoch anchor, if on an epoch boundary:
        if let Some(epoch_root) = compact_block.epoch_root {
            let epoch_duration = self.get_epoch_duration().await?;
            let index = Epoch::from_height(height, epoch_duration).index;
            self.set_nct_epoch_anchor(index, epoch_root);
        }

        // Write the CompactBlock:
        self.set_compact_block(compact_block);

        // Write the note commitment tree:
        self.stub_put_note_commitment_tree(&nct);

        Ok(())
    }

    /// Get the current block height.
    async fn height(&self) -> u64 {
        self.get_block_height()
            .await
            .expect("block height must be set")
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
