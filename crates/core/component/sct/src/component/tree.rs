use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{DomainType as _, StateReadProto, StateWriteProto};
use penumbra_sdk_tct as tct;
use tct::builder::{block, epoch};
use tracing::instrument;

use crate::{
    component::clock::EpochRead, event, state_key, CommitmentSource, NullificationInfo, Nullifier,
};

#[async_trait]
/// Provides read access to the state commitment tree and related data.
pub trait SctRead: StateRead {
    /// Fetch the state commitment tree from nonverifiable storage, preferring the cached tree if
    /// it exists.
    async fn get_sct(&self) -> tct::Tree {
        // If we have a cached tree, use that.
        if let Some(tree) = self.object_get(state_key::cache::cached_state_commitment_tree()) {
            return tree;
        }

        match self
            .nonverifiable_get_raw(state_key::tree::state_commitment_tree().as_bytes())
            .await
            .expect("able to retrieve state commitment tree from nonverifiable storage")
        {
            Some(bytes) => bincode::deserialize(&bytes).expect(
                "able to deserialize stored state commitment tree from nonverifiable storage",
            ),
            None => tct::Tree::new(),
        }
    }

    /// Return the SCT root for the given height, if it exists.
    /// If the height is not found, return `None`.
    async fn get_anchor_by_height(&self, height: u64) -> Result<Option<tct::Root>> {
        self.get(&state_key::tree::anchor_by_height(height)).await
    }

    /// Return metadata on the specified nullifier, if it has been spent.
    async fn spend_info(&self, nullifier: Nullifier) -> Result<Option<NullificationInfo>> {
        self.get(&state_key::nullifier_set::spent_nullifier_lookup(
            &nullifier,
        ))
        .await
    }

    /// Return the set of nullifiers that have been spent in the current block.
    fn pending_nullifiers(&self) -> im::Vector<Nullifier> {
        self.object_get(state_key::nullifier_set::pending_nullifiers())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> SctRead for T {}

#[async_trait]
/// Provides write access to the state commitment tree and related data.
pub trait SctManager: StateWrite {
    /// Write an SCT instance to nonverifiable storage and record
    /// the block and epoch roots in the JMT.
    ///
    /// # Panics
    /// If the epoch has not been set, or if a serialization failure occurs.
    async fn write_sct(
        &mut self,
        height: u64,
        sct: tct::Tree,
        block_root: block::Root,
        epoch_root: Option<epoch::Root>,
    ) {
        let sct_anchor = sct.root();
        let block_timestamp = self
            .get_current_block_timestamp()
            .await
            .map(|t| t.unix_timestamp())
            .unwrap_or(0);

        // Write the anchor as a key, so we can check claimed anchors...
        self.put_proto(state_key::tree::anchor_lookup(sct_anchor), height);
        // ... and as a value, so we can check SCT consistency.
        // TODO: can we move this out to NV storage?
        self.put(state_key::tree::anchor_by_height(height), sct_anchor);

        self.record_proto(event::anchor(height, sct_anchor, block_timestamp));
        self.record_proto(
            event::EventBlockRoot {
                height,
                root: block_root,
                timestamp_seconds: block_timestamp,
            }
            .to_proto(),
        );
        // Only record an epoch root event if we are ending the epoch.
        if let Some(epoch_root) = epoch_root {
            let index = self
                .get_current_epoch()
                .await
                .expect("epoch must be set")
                .index;
            self.record_proto(
                event::EventEpochRoot {
                    index,
                    root: epoch_root,
                    timestamp_seconds: block_timestamp,
                }
                .to_proto(),
            );
        }

        self.write_sct_cache(sct);
        self.persist_sct_cache();
    }

    /// Add a state commitment into the SCT, emitting an event recording its
    /// source, and return the insert position in the tree.
    async fn add_sct_commitment(
        &mut self,
        commitment: tct::StateCommitment,
        source: CommitmentSource,
    ) -> Result<tct::Position> {
        // Record in the SCT
        let mut tree = self.get_sct().await;
        let position = tree.insert(tct::Witness::Forget, commitment)?;
        self.write_sct_cache(tree);

        // Record the commitment source in an event
        self.record_proto(event::commitment(commitment, position, source));

        Ok(position)
    }

    #[instrument(skip(self, source))]
    /// Record a nullifier as spent in the verifiable storage.
    async fn nullify(&mut self, nullifier: Nullifier, source: CommitmentSource) {
        tracing::debug!("marking as spent");

        // We need to record the nullifier as spent in the JMT (to prevent
        // double spends), as well as in the CompactBlock (so that clients
        // can learn that their note was spent).
        self.put(
            state_key::nullifier_set::spent_nullifier_lookup(&nullifier),
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
        self.object_put(state_key::nullifier_set::pending_nullifiers(), nullifiers);
    }

    /// Seal the current block in the SCT, and produce an epoch root if
    /// we are ending an epoch as well.
    ///
    /// # Panics
    /// This method panic if the block is full, or if a serialization failure occurs.
    async fn end_sct_block(
        &mut self,
        end_epoch: bool,
    ) -> Result<(block::Root, Option<epoch::Root>)> {
        let height = self.get_block_height().await?;

        let mut tree = self.get_sct().await;

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

    // Set the state commitment tree in memory, but without committing to it in the nonverifiable
    // storage (very cheap).
    fn write_sct_cache(&mut self, tree: tct::Tree) {
        self.object_put(state_key::cache::cached_state_commitment_tree(), tree);
    }

    /// Persist the object-store SCT instance to nonverifiable storage.
    /// Note that this doesn't actually persist the SCT to disk, see the
    /// cndiarium documentation for more information.
    ///  
    /// # Panics
    /// This method panics if a serialization failure occurs.
    fn persist_sct_cache(&mut self) {
        // If the cached tree is dirty, flush it to storage
        if let Some(tree) =
            self.object_get::<tct::Tree>(state_key::cache::cached_state_commitment_tree())
        {
            let bytes = bincode::serialize(&tree)
                .expect("able to serialize state commitment tree to bincode");
            self.nonverifiable_put_raw(
                state_key::tree::state_commitment_tree().as_bytes().to_vec(),
                bytes,
            );
        }
    }
}

impl<T: StateWrite + ?Sized> SctManager for T {}

#[async_trait]
pub trait VerificationExt: StateRead {
    async fn check_claimed_anchor(&self, anchor: tct::Root) -> Result<()> {
        if anchor.is_empty() {
            return Ok(());
        }

        if let Some(anchor_height) = self
            .get_proto::<u64>(&state_key::tree::anchor_lookup(anchor))
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
            .get::<NullificationInfo>(&state_key::nullifier_set::spent_nullifier_lookup(
                &nullifier,
            ))
            .await?
        {
            anyhow::bail!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                hex::encode(info.id),
            );
        }
        Ok(())
    }
}

impl<T: StateRead + ?Sized> VerificationExt for T {}
