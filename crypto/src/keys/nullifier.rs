use crate::{
    merkle::Position,
    note,
    nullifier::{Nullifier, NULLIFIER_DOMAIN_SEP},
    Fq,
};
use poseidon377::hash_3;

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a note.
#[derive(Clone, Copy, Debug)]
pub struct NullifierKey(pub Fq);

impl NullifierKey {
    pub fn derive_nullifier(&self, pos: Position, note_commitment: &note::Commitment) -> Nullifier {
        Nullifier(hash_3(
            &NULLIFIER_DOMAIN_SEP,
            (self.0, note_commitment.0, (u64::from(pos)).into()),
        ))
    }
}
