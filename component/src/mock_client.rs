use std::collections::BTreeMap;

use penumbra_chain::{CompactBlock, Epoch, StatePayload};
use penumbra_crypto::{dex::swap::SwapPlaintext, note, FullViewingKey, Note};
use penumbra_storage::StateRead;
use penumbra_tct as tct;

use crate::shielded_pool::StateReadExt as _;

/// A bare-bones mock client for use exercising the state machine.
pub struct MockClient {
    latest_height: u64,
    epoch_duration: u64,
    fvk: FullViewingKey,
    notes: BTreeMap<note::Commitment, Note>,
    swaps: BTreeMap<tct::Commitment, SwapPlaintext>,
    sct: penumbra_tct::Tree,
}

impl MockClient {
    pub fn new(fvk: FullViewingKey, epoch_duration: u64) -> MockClient {
        Self {
            latest_height: u64::MAX,
            fvk,
            epoch_duration,
            notes: Default::default(),
            sct: Default::default(),
            swaps: Default::default(),
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
            let (latest_height, root) = self.latest_height_and_sct_root();
            anyhow::ensure!(latest_height == height, "latest height should be updated");
            let expected_root = state
                .anchor_by_height(height)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing sct anchor for height {}", height))?;
            anyhow::ensure!(
                root == expected_root,
                "client sct root should match chain state"
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
                            self.sct.insert(Keep, payload.note_commitment)?;
                        }
                        None => {
                            self.sct.insert(Forget, payload.note_commitment)?;
                        }
                    }
                }
                StatePayload::Swap { swap: payload, .. } => {
                    match payload.trial_decrypt(&self.fvk) {
                        Some(swap) => {
                            self.sct.insert(Keep, payload.commitment)?;
                            // At this point, we need to retain the swap plaintext,
                            // and also derive the expected output notes so we can
                            // notice them while scanning later blocks.
                            self.swaps.insert(payload.commitment, swap.clone());

                            let batch_data =
                                block.swap_outputs.get(&swap.trading_pair).ok_or_else(|| {
                                    anyhow::anyhow!("server gave invalid compact block")
                                })?;

                            let (output_1, output_2) = swap.output_notes(batch_data);
                            // Pre-insert the output notes into our notes table, so that
                            // we can notice them when we scan the block where they are claimed.
                            self.notes.insert(output_1.commit(), output_1);
                            self.notes.insert(output_2.commit(), output_2);
                        }
                        None => {
                            self.sct.insert(Forget, payload.commitment)?;
                        }
                    }
                }
                StatePayload::RolledUp(commitment) => {
                    if self.notes.contains_key(&commitment) {
                        // This is a note we anticipated, so retain its auth path.
                        self.sct.insert(Keep, commitment)?;
                    } else {
                        // This is someone else's note.
                        self.sct.insert(Forget, commitment)?;
                    }
                }
                StatePayload::Position { .. } => todo!(),
            }
        }
        self.sct.end_block()?;
        if Epoch::from_height(block.height, self.epoch_duration).is_epoch_end(block.height) {
            self.sct.end_epoch()?;
        }

        self.latest_height = block.height;

        Ok(())
    }

    pub fn latest_height_and_sct_root(&self) -> (u64, penumbra_tct::Root) {
        (self.latest_height, self.sct.root())
    }

    pub fn note_by_commitment(&self, commitment: &note::Commitment) -> Option<Note> {
        self.notes.get(commitment).cloned()
    }

    pub fn swap_by_commitment(&self, commitment: &note::Commitment) -> Option<SwapPlaintext> {
        self.swaps.get(commitment).cloned()
    }

    pub fn witness(&self, commitment: note::Commitment) -> Option<penumbra_tct::Proof> {
        self.sct.witness(commitment)
    }
}
