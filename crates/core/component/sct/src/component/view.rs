use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_tct as tct;
use tct::builder::{block, epoch};
use tracing::instrument;

use crate::{
    epoch::Epoch, event, params::SctParameters, state_key, CommitmentSource, NullificationInfo,
    Nullifier,
};

/// A helper trait for placing a `CommitmentSource` as ambient context during execution.
#[async_trait]
pub trait SourceContext: StateWrite {
    fn put_current_source(&mut self, source: Option<CommitmentSource>) {
        if let Some(source) = source {
            self.object_put(state_key::current_source(), source)
        } else {
            self.object_delete(state_key::current_source())
        }
    }

    fn get_current_source(&self) -> Option<CommitmentSource> {
        self.object_get(state_key::current_source())
    }

    /// Sets a mock source, for testing.
    ///
    /// The `counter` field allows distinguishing hashes at different stages of the test.
    fn put_mock_source(&mut self, counter: u8) {
        self.put_current_source(Some(CommitmentSource::Transaction {
            id: Some([counter; 32]),
        }))
    }
}

impl<T: StateWrite + ?Sized> SourceContext for T {}

#[async_trait]
/// Provides read access to the block eights, epoch, and other related data.
pub trait EpochRead: StateRead {
    /// Get the current block height.
    async fn get_block_height(&self) -> Result<u64> {
        self.get_proto(state_key::block_manager::block_height())
            .await?
            .ok_or_else(|| anyhow!("Missing block_height"))
    }

    /// Gets the current block timestamp from the JMT
    async fn get_block_timestamp(&self) -> Result<tendermint::Time> {
        let timestamp_string: String = self
            .get_proto(state_key::block_manager::block_timestamp())
            .await?
            .ok_or_else(|| anyhow!("Missing block_timestamp"))?;

        Ok(tendermint::Time::from_str(&timestamp_string)
            .context("block_timestamp was an invalid RFC3339 time string")?)
    }

    /// Get the current epoch.
    async fn current_epoch(&self) -> Result<Epoch> {
        // Get the height
        let height = self.get_block_height().await?;

        self.get(&state_key::epoch_manager::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for current height: {height}"))
    }

    async fn epoch_by_height(&self, height: u64) -> Result<Epoch> {
        self.get(&state_key::epoch_manager::epoch_by_height(height))
            .await?
            .ok_or_else(|| anyhow!("missing epoch for height"))
    }

    // Returns true if the epoch is ending early this block.
    fn epoch_ending_early(&self) -> bool {
        self.object_get(state_key::epoch_manager::end_epoch_early())
            .unwrap_or(false)
    }

    /// Gets the epoch duration for the chain (in blocks).
    async fn get_epoch_duration(&self) -> Result<u64> {
        self.get_sct_params()
            .await
            .map(|params| params.epoch_duration)
    }
}

impl<T: StateRead + ?Sized> EpochRead for T {}

/// This trait provides read access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the fee parameters from the JMT.
    async fn get_sct_params(&self) -> Result<SctParameters> {
        self.get(state_key::sct_params())
            .await?
            .ok_or_else(|| anyhow!("Missing SctParameters"))
    }
    /// Indicates if the sct parameters have been updated in this block.
    fn sct_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::sct_params_updated())
            .is_some()
    }

    async fn state_commitment_tree(&self) -> tct::Tree {
        // If we have a cached tree, use that.
        if let Some(tree) = self.object_get(state_key::cached_state_commitment_tree()) {
            return tree;
        }

        match self
            .nonverifiable_get_raw(state_key::state_commitment_tree().as_bytes())
            .await
            .expect("able to retrieve state commitment tree from nonverifiable storage")
        {
            Some(bytes) => bincode::deserialize(&bytes).expect(
                "able to deserialize stored state commitment tree from nonverifiable storage",
            ),
            None => tct::Tree::new(),
        }
    }

    async fn anchor_by_height(&self, height: u64) -> Result<Option<tct::Root>> {
        self.get(&state_key::anchor_by_height(height)).await
    }

    async fn check_claimed_anchor(&self, anchor: tct::Root) -> Result<()> {
        if anchor.is_empty() {
            return Ok(());
        }

        if let Some(anchor_height) = self
            .get_proto::<u64>(&state_key::anchor_lookup(anchor))
            .await?
        {
            tracing::debug!(?anchor, ?anchor_height, "anchor is valid");
            Ok(())
        } else {
            Err(anyhow!(
                "provided anchor {} is not a valid SCT root",
                anchor
            ))
        }
    }

    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(info) = self
            .get::<NullificationInfo>(&state_key::spent_nullifier_lookup(&nullifier))
            .await?
        {
            anyhow::bail!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                hex::encode(&info.id),
            );
        }
        Ok(())
    }

    async fn spend_info(&self, nullifier: Nullifier) -> Result<Option<NullificationInfo>> {
        self.get(&state_key::spent_nullifier_lookup(&nullifier))
            .await
    }

    fn pending_nullifiers(&self) -> im::Vector<Nullifier> {
        self.object_get(state_key::pending_nullifiers())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait SctManager: StateWrite {
    #[instrument(skip(self, source))]
    async fn nullify(&mut self, nullifier: Nullifier, source: CommitmentSource) {
        tracing::debug!("marking as spent");

        // We need to record the nullifier as spent in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so that clients
        // can learn that their note was spent).
        self.put(
            state_key::spent_nullifier_lookup(&nullifier),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            NullificationInfo {
                id: source
                    .id()
                    .expect("nullifiers are only consumed by transactions"),
                spend_height: self.get_block_height().await.expect("block height is set"),
            },
        );

        // Record the nullifier to be inserted into the compact block
        let mut nullifiers = self.pending_nullifiers();
        nullifiers.push_back(nullifier);
        self.object_put(state_key::pending_nullifiers(), nullifiers);
    }

    async fn add_sct_commitment(
        &mut self,
        commitment: tct::StateCommitment,
        source: CommitmentSource,
    ) -> Result<tct::Position> {
        // Record in the SCT
        let mut tree = self.state_commitment_tree().await;
        let position = tree.insert(tct::Witness::Forget, commitment)?;
        self.put_state_commitment_tree(tree);

        // Record the commitment source in an event
        self.record_proto(event::commitment(commitment, position, source));

        Ok(position)
    }

    async fn end_sct_block(
        &mut self,
        end_epoch: bool,
    ) -> Result<(block::Root, Option<epoch::Root>)> {
        let height = self.get_block_height().await?;

        let mut tree = self.state_commitment_tree().await;

        // Close the block in the SCT
        let block_root = tree
            .end_block()
            .expect("ending a block in the state commitment tree can never fail");

        // If the block ends an epoch, also close the epoch in the SCT
        let epoch_root = if end_epoch {
            let epoch_root = tree
                .end_epoch()
                .expect("ending an epoch in the state commitment tree can never fail");
            Some(epoch_root)
        } else {
            None
        };

        self.write_sct(height, tree, block_root, epoch_root).await;

        Ok((block_root, epoch_root))
    }
}

impl<T: StateWrite + ?Sized> SctManager for T {}

/// This trait provides write access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
trait StateWriteExt: StateWrite {
    /* TODO(erwan): move this to a dedicated trait */
    // Signals that the epoch should end this block.
    fn signal_end_epoch(&mut self) {
        self.object_put(state_key::epoch_manager::end_epoch_early(), true)
    }

    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) {
        self.put_proto(state_key::block_manager::block_height().to_string(), height)
    }

    /// Writes the epoch for the current height
    fn put_epoch_by_height(&mut self, height: u64, epoch: Epoch) {
        self.put(state_key::epoch_manager::epoch_by_height(height), epoch)
    }

    /* ***************************** */

    // Set the state commitment tree in memory, but without committing to it in the nonverifiable
    // storage (very cheap).
    fn put_state_commitment_tree(&mut self, tree: tct::Tree) {
        self.object_put(state_key::cached_state_commitment_tree(), tree);
    }

    // Serialize the current state commitment tree to storage (slightly more expensive, should only
    // happen once a block).
    async fn write_state_commitment_tree(&mut self) {
        // If the cached tree is dirty, flush it to storage
        if let Some(tree) = self.object_get::<tct::Tree>(state_key::cached_state_commitment_tree())
        {
            let bytes = bincode::serialize(&tree)
                .expect("able to serialize state commitment tree to bincode");
            self.nonverifiable_put_raw(
                state_key::state_commitment_tree().as_bytes().to_vec(),
                bytes,
            );
        }
    }

    async fn write_sct(
        &mut self,
        height: u64,
        sct: tct::Tree,
        block_root: block::Root,
        epoch_root: Option<epoch::Root>,
    ) {
        let sct_anchor = sct.root();

        // Write the anchor as a key, so we can check claimed anchors...
        self.put_proto(state_key::anchor_lookup(sct_anchor), height);
        // ... and as a value, so we can check SCT consistency.
        // TODO: can we move this out to NV storage?
        self.put(state_key::anchor_by_height(height), sct_anchor);

        self.record_proto(event::anchor(height, sct_anchor));
        self.record_proto(event::block_root(height, block_root));
        // Only record an epoch root event if we are ending the epoch.
        if let Some(epoch_root) = epoch_root {
            let index = self.current_epoch().await.expect("epoch must be set").index;
            self.record_proto(event::epoch_root(index, epoch_root));
        }

        self.put_state_commitment_tree(sct);
        self.write_state_commitment_tree().await;
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait EpochManager: StateWrite {
    /// Writes the block timestamp to the JMT
    fn put_block_timestamp(&mut self, timestamp: tendermint::Time) {
        self.put_proto(
            state_key::block_manager::block_timestamp().into(),
            timestamp.to_rfc3339(),
        )
    }

    // Signals that the epoch should end this block.
    fn signal_end_epoch(&mut self) {
        self.object_put(state_key::epoch_manager::end_epoch_early(), true)
    }

    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) {
        self.put_proto(state_key::block_manager::block_height().to_string(), height)
    }

    /// Writes the epoch for the current height
    fn put_epoch_by_height(&mut self, height: u64, epoch: Epoch) {
        self.put(state_key::epoch_manager::epoch_by_height(height), epoch)
    }
}

impl<T: StateWrite + ?Sized> EpochManager for T {}
