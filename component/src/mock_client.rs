use std::collections::BTreeMap;

use penumbra_chain::{CompactBlock, Epoch, StatePayload};
use penumbra_crypto::{note, FullViewingKey, Note};
use penumbra_storage::StateRead;

use crate::shielded_pool::StateReadExt as _;

/// A bare-bones mock client for use exercising the state machine.
pub struct MockClient {
    latest_height: u64,
    epoch_duration: u64,
    fvk: FullViewingKey,
    notes: BTreeMap<note::Commitment, Note>,
    nct: penumbra_tct::Tree,
}

impl MockClient {
    pub fn new(fvk: FullViewingKey, epoch_duration: u64) -> MockClient {
        Self {
            latest_height: u64::MAX,
            fvk,
            epoch_duration,
            notes: Default::default(),
            nct: Default::default(),
        }
    }

    pub async fn sync_to<R: StateRead>(
        &mut self,
        target_height: u64,
        state: R,
    ) -> anyhow::Result<()> {
        for height in 0..=target_height {
            let compact_block = state
                .compact_block(height)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing compact block for height {}", height))?;
            self.scan_block(compact_block)?;
            let (latest_height, root) = self.latest_height_and_nct_root();
            anyhow::ensure!(latest_height == height, "latest height should be updated");
            let expected_root = state
                .anchor_by_height(height)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing nct anchor for height {}", height))?;
            anyhow::ensure!(
                root == expected_root,
                "client nct root should match chain state"
            );
        }
        Ok(())
    }

    pub fn scan_block(&mut self, block: CompactBlock) -> anyhow::Result<()> {
        use penumbra_tct::Witness::*;

        if self.latest_height.wrapping_add(1) != block.height {
            return Err(anyhow::anyhow!(
                "wrong block height {} for latest height {}",
                block.height,
                self.latest_height
            ));
        }

        for payload in block.state_payloads {
            match payload {
                StatePayload::Note { note: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(note) => {
                            self.notes.insert(payload.note_commitment, note.clone());
                            self.nct.insert(Keep, payload.note_commitment)?;
                        }
                        None => {
                            self.nct.insert(Forget, payload.note_commitment)?;
                        }
                    }
                }
                StatePayload::RolledUp(commitment) => {
                    self.nct.insert(Forget, commitment)?;
                }
                StatePayload::Swap { .. } => todo!(),
            }
        }
        self.nct.end_block()?;
        if Epoch::from_height(block.height, self.epoch_duration).is_epoch_end(block.height) {
            self.nct.end_epoch()?;
        }

        self.latest_height = block.height;

        Ok(())
    }

    pub fn latest_height_and_nct_root(&self) -> (u64, penumbra_tct::Root) {
        (self.latest_height, self.nct.root())
    }

    pub fn note_by_commitment(&self, commitment: &note::Commitment) -> Option<Note> {
        self.notes.get(commitment).cloned()
    }

    pub fn witness(&self, commitment: note::Commitment) -> Option<penumbra_tct::Proof> {
        self.nct.witness(commitment)
    }
}
