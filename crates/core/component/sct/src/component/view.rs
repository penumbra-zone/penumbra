use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_tct as tct;
use tct::builder::{block, epoch};
use tracing::instrument;

// TODO: make epoch management the responsibility of this component

use crate::{event, state_key, CommitmentSource, NullificationInfo, Nullifier};

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
}

impl<T: StateWrite + ?Sized> SourceContext for T {}

/// This trait provides read access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateReadExt: StateRead {
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
            let index = self.epoch().await.expect("epoch must be set").index;
            self.record_proto(event::epoch_root(index, epoch_root));
        }

        self.put_state_commitment_tree(sct);
        self.write_state_commitment_tree().await;
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
