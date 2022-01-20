use std::collections::{BTreeMap, BTreeSet};

use decaf377::Fr;
use penumbra_crypto::{
    asset, ka,
    merkle::{Frontier, NoteCommitmentTree},
    note, Address, Fq, Note, Nullifier, Value, Zero,
};
use penumbra_stake::{
    BaseRateData, Epoch, FundingStream, IdentityKey, RateData, ValidatorStatus,
    STAKING_TOKEN_ASSET_ID,
};

use crate::verify::{NoteData, PositionedNoteData, VerifiedTransaction};

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
    /// If this is the last block of an epoch, validator statuses for the next epoch go here.
    pub next_validator_statuses: Option<Vec<ValidatorStatus>>,
    /// The net delegations performed in this block per validator.
    pub delegation_changes: BTreeMap<IdentityKey, i64>,
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
            next_validator_statuses: None,
            delegation_changes: BTreeMap::new(),
        }
    }

    /// We only get the height from ABCI in EndBlock, so this allows setting it in-place.
    pub fn set_height(&mut self, height: u64) -> Epoch {
        self.height = Some(height);
        let epoch = Epoch::from_height(height, self.epoch_duration);
        self.epoch = Some(epoch.clone());
        epoch
    }

    /// Adds a reward output from a validator's funding stream.
    /// currently we just construct a note with 0 blinding factor
    pub fn add_validator_reward_note(&mut self, amount: u64, destination: Address) {
        let val = Value {
            amount: amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        let note = Note::from_parts(
            *destination.diversifier(),
            *destination.transmission_key(),
            val,
            Fq::zero(),
        )
        .unwrap();
        let commitment = note.commit();
        let esk = ka::Secret::new_from_field(Fr::zero());
        let encrypted_note = note.encrypt(&esk);

        let note_data = NoteData {
            ephemeral_key: esk.public(),
            encrypted_note: encrypted_note,
            transaction_id: [0; 32],
        };

        self.note_commitment_tree.append(&commitment);

        let position = self
            .note_commitment_tree
            .bridges()
            .last()
            .map(|b| b.frontier().position().into())
            // If there are no bridges, the tree is empty
            .unwrap_or(0u64);

        self.notes.insert(
            commitment,
            PositionedNoteData {
                position,
                data: note_data,
            },
        );
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

        // Collect the nullifiers in this transaction
        for nullifier in transaction.spent_nullifiers {
            self.spent_nullifiers.insert(nullifier);
        }

        // Tally the delegation changes in this transaction
        for (identity_key, delegation_change) in transaction.delegation_changes {
            *self.delegation_changes.entry(identity_key).or_insert(0) += delegation_change;
        }
    }
}
