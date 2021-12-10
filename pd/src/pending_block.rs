use std::collections::{BTreeMap, BTreeSet};

use penumbra_crypto::{
    asset,
    merkle::{Frontier, NoteCommitmentTree, Tree},
    note, Nullifier,
};

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
}

impl PendingBlock {
    pub fn new(note_commitment_tree: NoteCommitmentTree) -> Self {
        Self {
            height: None,
            note_commitment_tree,
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            new_assets: BTreeMap::new(),
        }
    }

    /// We only get the height from ABCI in EndBlock, so this allows setting it in-place.
    pub fn set_height(&mut self, height: i64) {
        self.height = Some(height)
    }

    /// Adds the state changes from a verified transaction.
    pub fn add_transaction(&mut self, transaction: VerifiedTransaction) {
        for (note_commitment, data) in transaction.new_notes {
            tracing::debug!(?note_commitment, ?data);
            self.note_commitment_tree.append(&note_commitment);

            let (position, _) = self
                .note_commitment_tree
                .authentication_path(&note_commitment)
                .expect("we just appended this commitment");
            tracing::debug!(?position);

            self.notes.insert(
                note_commitment,
                PositionedNoteData {
                    position: u64::from(position),
                    data,
                },
            );
        }

        for nullifier in transaction.spent_nullifiers {
            self.spent_nullifiers.insert(nullifier);
        }
    }
}
