use crate::Component;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_chain::{
    genesis,
    quarantined::{self, Slashed},
    sync::{AnnotatedNotePayload, CompactBlock},
    Epoch, NoteSource, StateReadExt as _,
};
use penumbra_crypto::{asset, note, stake::IdentityKey, NotePayload, Nullifier, Value};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateTransaction, StateWrite};
use penumbra_tct as tct;
use tct::Tree;
use tendermint::abci;
use tracing::instrument;

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

        // Schedule all unquarantining that was set up in this block
        state.schedule_unquarantined_notes().await;

        // Handle any slashing that occurred in this block, unscheduling all affected notes and
        // nullifiers from future unbonding
        state.process_slashing().await;

        // Process all unquarantining scheduled for this block
        state.process_unquarantine().await;

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
            .get::<NoteSource, _>(&state_key::spent_nullifier_lookup(nullifier))
            .await?
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                source,
            ));
        }

        if let Some(source) = self
            .get::<NoteSource, _>(&state_key::quarantined_spent_nullifier_lookup(nullifier))
            .await?
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?} (currently quarantined)",
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

    async fn scheduled_to_apply(&self, epoch: u64) -> Result<quarantined::Scheduled> {
        Ok(self
            .get(&state_key::scheduled_to_apply(epoch))
            .await?
            .unwrap_or_default())
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

    // Returns whether the note was presently quarantined.
    // TODO: seems weird to return an Option type here given that it should never be None as-implemented
    async fn roll_back_note(&mut self, commitment: note::Commitment) -> Result<Option<NoteSource>> {
        // Get the note source of the note (or empty vec if already applied or rolled back)
        let source: NoteSource = self
            .get(&state_key::note_source(&commitment))
            .await?
            .expect("can't roll back note that was never created");

        // Delete the note from the set of all notes
        self.delete(state_key::note_source(&commitment));

        Ok(Some(source))
    }

    // Returns the source if the nullifier was in quarantine already
    // TODO: seems weird to return an Option type here given that it should never be None as-implemented
    #[instrument(skip(self))]
    async fn unquarantine_nullifier(&mut self, nullifier: Nullifier) -> Result<Option<NoteSource>> {
        tracing::debug!("removing quarantined nullifier");

        // Get the note source of the nullifier (or empty vec if already applied or rolled back)
        let source: NoteSource = self
            .get(&state_key::quarantined_spent_nullifier_lookup(nullifier))
            .await?
            .expect("can't unquarantine nullifier that was never quarantined");

        // Delete the nullifier from the quarantine set
        self.delete(state_key::quarantined_spent_nullifier_lookup(nullifier));

        Ok(Some(source))
    }
    async fn schedule_unquarantine(
        &mut self,
        epoch: u64,
        scheduled: quarantined::Scheduled,
    ) -> Result<()> {
        let mut updated_quarantined = self.scheduled_to_apply(epoch).await?;
        updated_quarantined.extend(scheduled);
        self.put(state_key::scheduled_to_apply(epoch), updated_quarantined);
        Ok(())
    }

    // Unschedule the unquarantining of all notes and nullifiers for the given validator, in any
    // epoch which could possibly still be unbonding
    async fn unschedule_all_slashed(&mut self) -> Result<Vec<IdentityKey>> {
        let height = self.get_block_height().await?;
        let epoch_duration = self.get_epoch_duration().await?;
        let this_epoch = Epoch::from_height(height, epoch_duration);
        let unbonding_epochs = self.get_chain_params().await?.unbonding_epochs;

        // TODO: restore
        /*
        let slashed: Slashed = self
            .get(&state_key::slashed_validators(height))
            .await?
            .unwrap_or_default();
         */
        let slashed = Slashed::default();

        for epoch in this_epoch.index.saturating_sub(unbonding_epochs)..=this_epoch.index {
            let mut updated_scheduled = self.scheduled_to_apply(epoch).await?;
            for &identity_key in &slashed.validators {
                let unbonding = updated_scheduled.unschedule_validator(identity_key);
                // Now we also ought to remove these nullifiers and notes from quarantine without
                // applying them:
                for &nullifier in unbonding.nullifiers.iter() {
                    self.unquarantine_nullifier(nullifier).await?;
                }
                for note_payload in unbonding.note_payloads.iter() {
                    self.roll_back_note(note_payload.payload.note_commitment)
                        .await?;
                }
            }
            // We're removed all the scheduled notes and nullifiers for this epoch and identity key:
            self.put(state_key::scheduled_to_apply(epoch), updated_scheduled);
        }

        Ok(slashed.validators)
    }

    #[instrument(skip(self, source, payload), fields(note_commitment = ?payload.note_commitment))]
    async fn schedule_note(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        payload: NotePayload,
        source: NoteSource,
    ) {
        tracing::debug!("scheduling note");

        // 1. Record its source in the JMT
        self.put(state_key::note_source(&payload.note_commitment), source);

        // 2. Schedule it in the compact block
        // TODO: port this over, quarantine logic is very complicated,
        // but we have to replicate it exactly as-is, because it carries over
        // to the client side
        let mut compact_block = self.stub_compact_block();
        compact_block.quarantined.schedule_note(
            epoch,
            identity_key,
            AnnotatedNotePayload { payload, source },
        );
        self.stub_put_compact_block(compact_block);
    }

    #[instrument(skip(self, source))]
    async fn quarantined_spend_nullifier(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        nullifier: Nullifier,
        source: NoteSource,
    ) {
        // We need to record the nullifier as spent under quarantine in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so clients can learn their note
        // was provisionally spent, pending quarantine period).
        tracing::debug!("marking as spent (currently quarantined)");
        self.put(
            state_key::quarantined_spent_nullifier_lookup(nullifier),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            source,
        );
        // Queue up scheduling this nullifier to be unquarantined: the actual state-writing
        // for all quarantined nullifiers happens during end_block, to avoid state churn
        let mut compact_block = self.stub_compact_block();
        compact_block
            .quarantined
            .schedule_nullifier(epoch, identity_key, nullifier);
        self.stub_put_compact_block(compact_block);
    }

    /// Get the current block height.
    async fn height(&self) -> u64 {
        self.get_block_height()
            .await
            .expect("block height must be set")
    }

    async fn schedule_unquarantined_notes(&mut self) {
        // First, we group all the scheduled quarantined notes by unquarantine epoch, in the process
        // resetting the quarantine field of the component
        let compact_block = self.stub_compact_block();
        for (&unbonding_epoch, scheduled) in compact_block.quarantined.iter() {
            self.schedule_unquarantine(unbonding_epoch, scheduled.clone())
                .await
                .expect("scheduling unquarantine must succeed");
        }
    }

    // Process any slashing that occrred in this block.
    async fn process_slashing(&mut self) {
        let mut compact_block = self.stub_compact_block();
        compact_block.slashed.extend(
            self.unschedule_all_slashed()
                .await
                .expect("can unschedule slashed"),
        );
        self.stub_put_compact_block(compact_block);
    }

    // Process any notes/nullifiers due to be unquarantined in this block, if it's an
    // epoch-ending block
    #[instrument(skip(self))]
    async fn process_unquarantine(&mut self) {
        let this_epoch = self.epoch().await.unwrap();

        if this_epoch.is_epoch_end(self.height().await) {
            for (_, per_validator) in self
                .scheduled_to_apply(this_epoch.index)
                .await
                .expect("can look up quarantined for this epoch")
            {
                // For all the note payloads scheduled for unquarantine now, remove them from
                // quarantine and add them to the proper notes for this block
                for note_payload in per_validator.note_payloads {
                    tracing::debug!(?note_payload, "unquarantining note");
                    self.add_note(note_payload).await;
                }
                // For all the nullifiers scheduled for unquarantine now, remove them from
                // quarantine and add them to the proper nullifiers for this block
                for nullifier in per_validator.nullifiers {
                    let note_source = self
                        .unquarantine_nullifier(nullifier)
                        .await
                        .expect("can try to unquarantine nullifier")
                        .expect("nullifier to unquarantine has source");
                    tracing::debug!(?nullifier, "unquarantining nullifier");
                    self.spend_nullifier(nullifier, note_source).await;
                }
            }
        }
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
