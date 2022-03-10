//! The tiered commitment tree for Penumbra.

// Cargo doc complains if the recursion limit isn't higher, even though cargo build succeeds:
#![recursion_limit = "256"]
#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

use std::fmt::Debug;

pub mod internal;

#[doc(inline)]
pub use internal::{
    active::Insert,
    hash::Hash,
    path::AuthPath,
    proof::{Proof, ProveError, VerifiedProof, VerifyError},
};

#[allow(unused_imports)]
use internal::{
    active::{Active, Focus, Full, Tier},
    complete::Complete,
    hash::GetHash,
    height::Height,
    item::Item,
    Witness,
};

pub use ark_ff::fields::PrimeField;

/// A commitment to be stored in the tree, as an element of the base field of the curve used by the
/// Poseidon hash function instantiated for BLS12-377. If you want to witness this commitment in a
/// tree, insert it using [`Insert::Keep`](crate::Insert::Keep).
pub use poseidon377::Fq;

mod eternity;
pub use eternity::Eternity;

mod epoch;
pub use epoch::Epoch;

mod block;
pub use block::Block;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Eternity, [u8; 88]);
    }

    #[test]
    fn check_eternity_proof_size() {
        static_assertions::assert_eq_size!(Proof<Eternity>, [u8; 2344]);
    }
}
