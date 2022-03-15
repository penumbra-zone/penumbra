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
    proof::{Proof, VerifiedProof, VerifyError},
};

#[allow(unused_imports)]
use internal::{
    active::{Active, Focus, Forget, Full, Item, Tier},
    complete::{Complete, ForgetOwned},
    hash::GetHash,
    height::Height,
    index,
    path::AuthPath,
    Witness,
};

/// A commitment to be stored in the tree, as an element of the base field of the curve used by the
/// Poseidon hash function instantiated for BLS12-377. If you want to witness this commitment in a
/// tree, insert it using [`Insert::Keep`](crate::Insert::Keep).
pub use poseidon377::Fq;

mod eternity;
pub use eternity::{Block, Epoch, Eternity};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Eternity, [u8; 96]);
    }

    #[test]
    fn check_eternity_proof_size() {
        static_assertions::assert_eq_size!(Proof<Eternity>, [u8; 2344]);
    }
}
