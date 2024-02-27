use anyhow::Error;
use cnidarium::StateRead;
use penumbra_compact_block::{component::StateReadExt as _, CompactBlock, StatePayload};
use penumbra_dex::swap::SwapPlaintext;
use penumbra_keys::{keys::SpendKey, FullViewingKey};
use penumbra_sct::component::{clock::EpochRead, tree::SctRead};
use penumbra_shielded_pool::{note, Note};
use penumbra_tct as tct;
use penumbra_transaction::{AuthorizationData, Transaction, TransactionPlan, WitnessData};
use rand_core::OsRng;
use std::collections::BTreeMap;

/// A bare-bones mock client for use exercising the state machine.
pub struct MockClient {
    latest_height: u64,
    sk: SpendKey,
    pub fvk: FullViewingKey,
    pub notes: BTreeMap<note::StateCommitment, Note>,
    swaps: BTreeMap<tct::StateCommitment, SwapPlaintext>,
    pub sct: penumbra_tct::Tree,
}

impl MockClient {
    pub fn new(sk: SpendKey) -> MockClient {
        Self {
            latest_height: u64::MAX,
            fvk: sk.full_viewing_key().clone(),
            sk,
            notes: Default::default(),
            sct: Default::default(),
            swaps: Default::default(),
        }
    }

    pub async fn with_sync_to_storage(
        mut self,
        storage: impl AsRef<cnidarium::Storage>,
    ) -> anyhow::Result<Self> {
        let latest = storage.as_ref().latest_snapshot();
        self.sync_to_latest(latest).await?;

        Ok(self)
    }

    pub async fn sync_to_latest<R: StateRead>(&mut self, state: R) -> anyhow::Result<()> {
        let height = state.get_block_height().await?;
        self.sync_to(height, state).await?;
        Ok(())
    }

    pub async fn sync_to<R: StateRead>(
        &mut self,
        target_height: u64,
        state: R,
    ) -> anyhow::Result<()> {
        let start_height = self.latest_height.wrapping_add(1);
        for height in start_height..=target_height {
            let compact_block = state
                .compact_block(height)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing compact block for height {}", height))?;
            self.scan_block(compact_block)?;
            let (latest_height, root) = self.latest_height_and_sct_root();
            anyhow::ensure!(latest_height == height, "latest height should be updated");
            let expected_root = state
                .get_anchor_by_height(height)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing sct anchor for height {}", height))?;
            anyhow::ensure!(
                root == expected_root,
                format!(
                    "client sct root should match chain state: {:?} != {:?}",
                    root, expected_root
                )
            );
        }
        Ok(())
    }

    pub fn scan_block(&mut self, block: CompactBlock) -> anyhow::Result<()> {
        use penumbra_tct::Witness::*;

        if self.latest_height.wrapping_add(1) != block.height {
            anyhow::bail!(
                "wrong block height {} for latest height {}",
                block.height,
                self.latest_height
            );
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
                StatePayload::RolledUp { commitment, .. } => {
                    if self.notes.contains_key(&commitment) {
                        // This is a note we anticipated, so retain its auth path.
                        self.sct.insert(Keep, commitment)?;
                    } else {
                        // This is someone else's note.
                        self.sct.insert(Forget, commitment)?;
                    }
                }
            }
        }
        self.sct.end_block()?;
        if block.epoch_root.is_some() {
            self.sct.end_epoch()?;
        }

        self.latest_height = block.height;

        Ok(())
    }

    pub fn latest_height_and_sct_root(&self) -> (u64, penumbra_tct::Root) {
        (self.latest_height, self.sct.root())
    }

    pub fn note_by_commitment(&self, commitment: &note::StateCommitment) -> Option<Note> {
        self.notes.get(commitment).cloned()
    }

    pub fn swap_by_commitment(&self, commitment: &note::StateCommitment) -> Option<SwapPlaintext> {
        self.swaps.get(commitment).cloned()
    }

    pub fn position(&self, commitment: note::StateCommitment) -> Option<penumbra_tct::Position> {
        self.sct.witness(commitment).map(|proof| proof.position())
    }

    pub fn witness_commitment(
        &self,
        commitment: note::StateCommitment,
    ) -> Option<penumbra_tct::Proof> {
        self.sct.witness(commitment)
    }

    pub fn witness_plan(&self, plan: &TransactionPlan) -> Result<WitnessData, Error> {
        Ok(WitnessData {
            anchor: self.sct.root(),
            // TODO: this will only witness spends, not other proofs like swaps
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    let nc = spend.note.commit();
                    Ok((
                        nc,
                        self.sct.witness(nc).ok_or_else(|| {
                            anyhow::anyhow!("note commitment {:?} unknown to client", nc)
                        })?,
                    ))
                })
                .collect::<Result<_, Error>>()?,
        })
    }

    pub fn authorize_plan(&self, plan: &TransactionPlan) -> Result<AuthorizationData, Error> {
        plan.authorize(OsRng, &self.sk)
    }

    pub async fn witness_auth_build(&self, plan: &TransactionPlan) -> Result<Transaction, Error> {
        let witness_data = self.witness_plan(plan)?;
        let auth_data = self.authorize_plan(plan)?;
        plan.clone()
            .build_concurrent(&self.fvk, &witness_data, &auth_data)
            .await
    }
}
