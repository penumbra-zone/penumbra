use std::collections::{BTreeMap, BTreeSet};

use penumbra_crypto::{ka, merkle, note, Nullifier};
use penumbra_stake::{
    Delegate, IdentityKey, Undelegate, ValidatorDefinition, VerifiedValidatorDefinition,
};

mod stateful;
mod stateless;

// TODO: eliminate (#374)
pub use stateless::StatelessTransactionExt;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct NoteData {
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
    pub transaction_id: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct PositionedNoteData {
    pub position: u64,
    pub data: NoteData,
}

/// `PendingTransaction` holds data after stateless checks have been applied.
/// TODO this is a bad name
#[derive(Debug)]
pub struct PendingTransaction {
    /// Transaction ID.
    pub id: [u8; 32],
    /// Root of the note commitment tree.
    pub root: merkle::Root,
    /// Note data to add from outputs in this transaction.
    pub new_notes: BTreeMap<note::Commitment, NoteData>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Delegations performed in this transaction.
    pub delegations: Vec<Delegate>,
    /// Undelegation, if any, performed in this transaction (there must be no more than one).
    pub undelegation: Option<Undelegate>,
    /// Validator definitions received in the transaction.
    pub validator_definitions: Vec<ValidatorDefinition>,
}

/// `VerifiedTransaction` represents a transaction after all checks have passed.
/// TODO this is a bad name
#[derive(Debug)]
pub struct VerifiedTransaction {
    /// Transaction ID.
    pub id: [u8; 32],
    /// Note data to add from outputs in this transaction.
    pub new_notes: BTreeMap<note::Commitment, NoteData>,
    /// List of spent nullifiers from spends in this transaction.
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Net delegations performed in this transaction.
    ///
    /// An identity key mapped to zero is different from an identity key that is absent; the former
    /// indicates that a validator's net change in delegation in this transaction was zero *but it
    /// experienced some (un)delegations*.
    pub delegation_changes: BTreeMap<IdentityKey, i64>,
    /// The validators from whom an undelegation was performed in this transaction.
    pub undelegation_validator: Option<IdentityKey>,
    /// Validator definitions received in the transaction.
    pub validator_definitions: Vec<VerifiedValidatorDefinition>,
}
