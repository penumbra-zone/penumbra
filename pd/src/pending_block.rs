use std::collections::{BTreeMap, BTreeSet};

use penumbra_crypto::{
    asset,
    merkle::{Frontier, NoteCommitmentTree},
    note, Nullifier,
};
use penumbra_stake::{BaseRateData, Epoch, RateData};

use crate::verify::{PositionedNoteData, VerifiedTransaction};

/// Stores pending state changes from transactions.
#[derive(Debug, Clone)]
pub struct PendingBlock {
    pub height: Option<u64>,
    pub note_commitment_tree: NoteCommitmentTree,
    /// Stores note commitments for convienience when updating the NCT.
    pub notes: BTreeMap<note::Commitment, PositionedNoteData>,
    /// Nullifiers that were spent in this block.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Records any updates to the token supply of some asset that happened in this block.
    pub supply_updates: BTreeMap<asset::Id, (asset::Denom, u64)>,
    /// Indicates the epoch the block belongs to.
    pub epoch: Option<Epoch>,
    /// Indicates the duration in blocks of each epoch.
    pub epoch_duration: u64,
    /// If this is the last block of an epoch, base rates for the next epoch go here.
    pub next_base_rate: Option<BaseRateData>,
    /// If this is the last block of an epoch, validator rates for the next epoch go here.
    pub next_rates: Option<Vec<RateData>>,
}

impl PendingBlock {
    pub fn new(note_commitment_tree: NoteCommitmentTree, epoch_duration: u64) -> Self {
        Self {
            height: None,
            note_commitment_tree,
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            supply_updates: BTreeMap::new(),
            epoch: None,
            epoch_duration,
            next_base_rate: None,
            next_rates: None,
        }
    }

    /// We only get the height from ABCI in EndBlock, so this allows setting it in-place.
    pub fn set_height(&mut self, height: u64) -> Epoch {
        self.height = Some(height);
        let epoch = Epoch::from_height(height, self.epoch_duration);
        self.epoch = Some(epoch.clone());
        epoch
    }

    /// Adds the state changes from a verified transaction.
    pub fn add_transaction(&mut self, transaction: VerifiedTransaction) {
        for (note_commitment, data) in transaction.new_notes {
            self.note_commitment_tree.append(&note_commitment);

            let position = self
                .note_commitment_tree
                .bridges()
                .last()
                .map(|b| b.frontier().position().into())
                // If there are no bridges, the tree is empty
                .unwrap_or(0u64);

            self.notes
                .insert(note_commitment, PositionedNoteData { position, data });
        }

        for nullifier in transaction.spent_nullifiers {
            self.spent_nullifiers.insert(nullifier);
        }
    }
}
