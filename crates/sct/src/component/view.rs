use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use penumbra_tct as tct;
use tct::builder::{block, epoch};

// TODO: make epoch management the responsibility of this component

use crate::state_key;

/// This trait provides read access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn state_commitment_tree(&self) -> tct::Tree {
        match self
            .nonconsensus_get_raw(state_key::stub_state_commitment_tree().as_bytes())
            .await
            .unwrap()
        {
            Some(bytes) => bincode::deserialize(&bytes).unwrap(),
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
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait SctManager: StateWrite {
    async fn add_sct_commitment(&mut self, commitment: tct::Commitment) -> Result<tct::Position> {
        let mut tree = self.state_commitment_tree().await;
        let position = tree.insert(tct::Witness::Forget, commitment)?;
        self.put_state_commitment_tree(&tree);
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
    fn put_state_commitment_tree(&mut self, tree: &tct::Tree) {
        let bytes = bincode::serialize(&tree).unwrap();
        self.nonconsensus_put_raw(
            state_key::stub_state_commitment_tree().as_bytes().to_vec(),
            bytes,
        );
    }

    fn set_sct_anchor(&mut self, height: u64, sct_anchor: tct::Root) {
        tracing::debug!(?height, ?sct_anchor, "writing anchor");

        self.put(state_key::anchor_by_height(height), sct_anchor);
        self.put_proto(state_key::anchor_lookup(sct_anchor), height);
    }

    fn set_sct_block_anchor(&mut self, height: u64, sct_block_anchor: block::Root) {
        tracing::debug!(?height, ?sct_block_anchor, "writing block anchor");

        self.put(state_key::block_anchor_by_height(height), sct_block_anchor);
        self.put_proto(state_key::block_anchor_lookup(sct_block_anchor), height);
    }

    fn set_sct_epoch_anchor(&mut self, index: u64, sct_block_anchor: epoch::Root) {
        tracing::debug!(?index, ?sct_block_anchor, "writing epoch anchor");

        self.put(state_key::epoch_anchor_by_index(index), sct_block_anchor);
        self.put_proto(state_key::epoch_anchor_lookup(sct_block_anchor), index);
    }

    async fn write_sct(
        &mut self,
        height: u64,
        sct: tct::Tree,
        block_root: block::Root,
        epoch_root: Option<epoch::Root>,
    ) {
        self.set_sct_anchor(height, sct.root());

        self.set_sct_block_anchor(height, block_root);

        if let Some(epoch_root) = epoch_root {
            let index = self.epoch().await.expect("epoch must be set").index;
            self.set_sct_epoch_anchor(index, epoch_root);
        }

        self.put_state_commitment_tree(&sct);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
