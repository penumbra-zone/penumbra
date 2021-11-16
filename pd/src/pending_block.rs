use std::collections::{BTreeMap, BTreeSet};

use penumbra_crypto::{
    action::output::{self, Output},
    action::spend::{self, Spend},
    ka,
    merkle::{Frontier, NoteCommitmentTree},
    note, Action, Nullifier, Transaction,
    asset
};

/// Stores pending state changes from transactions.
#[derive(Debug)]
pub struct PendingBlock {
    pub height: Option<i64>,
    pub note_commitment_tree: NoteCommitmentTree,
    /// Stores note commitments for convienience when updating the NCT.
    pub notes: BTreeMap<note::Commitment, NoteData>,
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

    /// Adds the changes from a transaction.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        let transaction_id = transaction.id();

        for action in transaction.transaction_body().actions {
            match action {
                Action::Output(Output {
                    body:
                        output::Body {
                            note_commitment,
                            ephemeral_key,
                            encrypted_note,
                            ..
                        },
                    ..
                }) => {
                    self.notes.insert(
                        note_commitment,
                        NoteData {
                            ephemeral_key,
                            encrypted_note,
                            transaction_id,
                        },
                    );
                    self.note_commitment_tree.append(&note_commitment);
                }
                Action::Spend(Spend {
                    body: spend::Body { nullifier, .. },
                    ..
                }) => {
                    self.spent_nullifiers.insert(nullifier);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct NoteData {
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
    pub transaction_id: [u8; 32],
}
