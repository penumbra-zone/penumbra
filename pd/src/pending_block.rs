use std::collections::{BTreeMap, BTreeSet};

use ark_ff::PrimeField;
use decaf377::Fr;
use penumbra_crypto::{
    ka,
    merkle::{Frontier, NoteCommitmentTree},
    note, Address, Fq, Note, Nullifier, One, Value,
};
use penumbra_stake::{Epoch, IdentityKey, STAKING_TOKEN_ASSET_ID};
use tracing::instrument;

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
    /// The counter containing the number of rewards notes in the epoch. we need this to keep the
    /// blinding factor of the reward notes unique.
    reward_counter: u64,
    /// Records all the quarantined inputs/outputs from this block.
    pub quarantine: Vec<QuarantineGroup>,
    /// Nullifiers to remove from the quarantined set when this block is committed, making their
    /// spend permanent.
    pub unbonding_nullifiers: BTreeSet<Nullifier>,
    /// Notes to be dropped from the quarantine set when this block is committed, reverting their spend.
    pub reverting_notes: BTreeSet<note::Commitment>,
    /// Nullifiers to remove from the nullifier set when this block is committed, reverting their spend.
    pub reverting_nullifiers: BTreeSet<Nullifier>,
    /// Indicates the epoch the block belongs to.
    pub epoch: Option<Epoch>,
}

/// A group of notes and nullifiers, all to be quarantined relative to a shared set of validators.
#[derive(Debug, Clone)]
pub struct QuarantineGroup {
    /// If this validator is slashed while the notes and nullifiers in this quarantined group, then
    /// all of the notes should be dropped and all the nullifiers removed from the NCT.
    pub validator_identity_key: IdentityKey,
    /// The set of notes in this group.
    pub notes: BTreeMap<note::Commitment, NoteData>,
    /// The set of nullifiers in this group.
    pub nullifiers: BTreeSet<Nullifier>,
}

impl PendingBlock {
    pub fn new(note_commitment_tree: NoteCommitmentTree) -> Self {
        Self {
            height: None,
            note_commitment_tree,
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            reward_counter: 0,
            quarantine: Vec::new(),
            reverting_notes: BTreeSet::new(),
            unbonding_nullifiers: BTreeSet::new(),
            reverting_nullifiers: BTreeSet::new(),
            epoch: None,
        }
    }

    /// We only get the height from ABCI in EndBlock, so this allows setting it in-place.
    pub fn set_height(&mut self, height: u64, epoch_duration: u64) -> Epoch {
        self.height = Some(height);
        let epoch = Epoch::from_height(height, epoch_duration);
        self.epoch = Some(epoch.clone());
        epoch
    }

    /// Adds a reward output for a validator's funding stream.
    #[instrument(skip(self, destination), fields(destination = %destination))]
    pub fn add_validator_reward_note(&mut self, amount: u64, destination: Address) {
        if amount == 0 {
            // Skip adding an empty note to the chain.
            return;
        }

        let val = Value {
            amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        let blinding_factor_input = blake2b_simd::Params::default()
            .personal(b"fundingstrm_note")
            .to_state()
            .update(&self.epoch.as_ref().unwrap().index.to_le_bytes())
            .update(&self.reward_counter.to_le_bytes())
            .finalize();

        let note = Note::from_parts(
            *destination.diversifier(),
            *destination.transmission_key(),
            val,
            Fq::from_le_bytes_mod_order(blinding_factor_input.as_bytes()),
        )
        .unwrap();
        let commitment = note.commit();

        tracing::debug!(?note, ?commitment);

        let esk = ka::Secret::new_from_field(Fr::one());
        let encrypted_note = note.encrypt(&esk);

        let note_data = NoteData {
            ephemeral_key: esk.diversified_public(&note.diversified_generator()),
            encrypted_note,
            transaction_id: [0; 32],
        };

        self.add_note(commitment, note_data);

        self.reward_counter += 1;
    }

    /// Adds a new note to this pending block.
    pub fn add_note(&mut self, commitment: note::Commitment, data: NoteData) {
        tracing::info!(?commitment, "adding note");

        self.note_commitment_tree.append(&commitment);

        let position = self
            .note_commitment_tree
            .bridges()
            .last()
            .map(|b| b.frontier().position().into())
            // If there are no bridges, the tree is empty
            .unwrap_or(0u64);

        self.notes
            .insert(commitment, PositionedNoteData { position, data });
    }

    /// Adds the state changes from a verified transaction.
    pub fn add_transaction(&mut self, transaction: VerifiedTransaction) {
        // Hack: skip quarantining, because we didn't implement the client-side logic.
        // if let Some(validator_identity_key) = transaction.undelegation_validator {
        //     // If a transaction contains an undelegation, we *do not insert any of its outputs*
        //     // into the NCT; instead we store them separately, to be inserted into the NCT only
        //     // after the unbonding period occurs.
        //     self.quarantine.push(QuarantineGroup {
        //         validator_identity_key,
        //         notes: transaction.new_notes.into_iter().collect(),
        //         nullifiers: transaction.spent_nullifiers.iter().cloned().collect(),
        //     });
        // } else {
        // If a transaction does not contain any undelegations, we insert its outputs
        // immediately into the NCT.
        for (commitment, data) in transaction.new_notes {
            self.add_note(commitment, data);
        }
        // }

        // Unconditionally, insert all nullifiers spent in this transaction into the spent set to
        // prevent double-spends, regardless of quarantine status.
        for nullifier in transaction.spent_nullifiers {
            self.spent_nullifiers.insert(nullifier);
        }
    }
}
