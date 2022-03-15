//! The tiered commitment tree for Penumbra.

// Cargo doc complains if the recursion limit isn't higher, even though cargo build succeeds:
#![recursion_limit = "256"]
#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

use std::fmt::Debug;

pub mod internal;

#[allow(unused_imports)]
use internal::{
    active::{Active, Focus, Full, Insert, Item, Tier},
    complete::{Complete, ForgetOwned},
    hash::GetHash,
    hash::Hash,
    height::Height,
    index,
    path::AuthPath,
    proof,
};

/// A commitment to be stored in a [`Block`].
///
/// This is an element of the base field of the curve used by the Poseidon hash function
/// instantiated for BLS12-377.
pub use poseidon377::Fq as Commitment;

mod eternity;
pub use eternity::{block, epoch, Block, Epoch, Eternity, Proof, Root, VerifiedProof, VerifyError};

/// When inserting an item into an [`Eternity`], [`Epoch`], or [`Block`], should we [`Keep`] it to
/// allow it to be witnessed later, or [`Forget`] about it after updating the root hash?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Witness {
    /// Keep this item so it can be witnessed later.
    Keep,
    /// Forget this item so it does not take up space, but it cannot be witnessed later.
    Forget,
}

pub use Witness::{Forget, Keep};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Eternity, [u8; 96]);
    }

    #[test]
    fn check_eternity_proof_size() {
        static_assertions::assert_eq_size!(Proof, [u8; 2344]);
    }
}
