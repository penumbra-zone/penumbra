use std::collections::{BTreeMap, BTreeSet};

use penumbra_crypto::{
    asset,
    merkle::{Frontier, NoteCommitmentTree},
    note, Nullifier,
};
use penumbra_stake::Epoch;

use crate::verify::{PositionedNoteData, VerifiedTransaction};

/// Stores pending state changes from transactions.
#[derive(Debug, Clone)]
pub struct PendingBlock {
    pub height: Option<i64>,
    pub note_commitment_tree: NoteCommitmentTree,
    /// Stores note commitments for convienience when updating the NCT.
    pub notes: BTreeMap<note::Commitment, PositionedNoteData>,
    /// Nullifiers that were spent in this block.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Stores new asset types found in this block that need to be added to the asset registry.
    pub new_assets: BTreeMap<asset::Id, String>,
    /// Indicates whether the Pending Block starts a new epoch.
    pub is_epoch_boundary: Option<bool>,
    /// Indicates the epoch the block belongs to.
    pub epoch: Option<Epoch>,
}

impl PendingBlock {
    pub fn new(note_commitment_tree: NoteCommitmentTree) -> Self {
        Self {
            height: None,
            note_commitment_tree,
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            new_assets: BTreeMap::new(),
            is_epoch_boundary: None,
            epoch: None,
        }
    }

    /// We only get the height from ABCI in EndBlock, so this allows setting it in-place.
    pub fn set_height_and_epoch(&mut self, height: i64, epoch: Epoch) {
        self.height = Some(height);
        self.epoch = Some(epoch)
    }

    /// Whether a pending block is an epoch boundary or not can only be determined in EndBlock.
    pub fn set_epoch_boundary(&mut self, is_epoch_boundary: bool) {
        self.is_epoch_boundary = Some(is_epoch_boundary)
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
