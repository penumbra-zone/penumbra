use poseidon377::hash_3;

use crate::{
    note::Commitment,
    nullifier::{Nullifier, NULLIFIER_DOMAIN_SEP},
    Fq,
};

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a positioned piece of state.
#[derive(Clone, Copy, Debug)]
pub struct NullifierKey(pub Fq);

impl NullifierKey {
    pub fn derive_nullifier(
        &self,
        pos: penumbra_tct::Position,
        state_commitment: &Commitment,
    ) -> Nullifier {
        Nullifier(hash_3(
            &NULLIFIER_DOMAIN_SEP,
            (self.0, state_commitment.0, (u64::from(pos)).into()),
        ))
    }
}
