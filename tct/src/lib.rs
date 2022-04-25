//! The tiered commitment tree for Penumbra.
//!
//! ```ascii,no_run
//! Eternity┃           ╱╲ ◀───────────── Anchor
//!     Tree┃          ╱││╲               = Eternity Root
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Eternity Leaf
//!                         ▲             = Epoch Root
//!                      ┌──┘
//!                      │
//!                      │
//!    Epoch┃           ╱╲ ◀───────────── Epoch Root
//!     Tree┃          ╱││╲
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Epoch Leaf
//!                  ▲                    = Block Root
//!                  └───┐
//!                      │
//!                      │
//!    Block┃           ╱╲ ◀───────────── Block Root
//!     Tree┃          ╱││╲
//!         ┃         * ** *           ╮
//!         ┃      *   *  *   *        │ 8 levels
//!         ┃   *     *    *     *     ╯
//!         ┃  ╱╲    ╱╲    ╱╲    ╱╲
//!         ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Block Leaf
//!                                       = Note Commitment
//! ```

// Cargo doc complains if the recursion limit isn't higher, even though cargo build succeeds:
#![recursion_limit = "256"]
#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

mod index;
mod serialize;

#[cfg(any(doc, feature = "internal"))]
pub mod internal;
#[cfg(not(any(doc, feature = "internal")))]
mod internal;

#[cfg(any(doc, test, feature = "spec"))]
pub mod spec;

use internal::{
    complete::{Complete, ForgetOwned},
    frontier::{Focus, Frontier, Insert, Item, Tier},
    hash::GetHash,
    hash::Hash,
    height::Height,
    path::AuthPath,
    proof,
};

#[doc(inline)]
pub use crate::internal::{
    path::PathDecodeError,
    proof::{ProofDecodeError, VerifyError},
};

/// A commitment to be stored in a [`Block`].
///
/// This is an element of the base field of the curve used by the Poseidon hash function
/// instantiated for BLS12-377.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Commitment(#[serde(with = "crate::serialize::fq")] pub poseidon377::Fq);

impl Debug for Commitment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use ark_ff::ToBytes;
        let mut bytes = Vec::with_capacity(4 * 8);
        self.0.write(&mut bytes).unwrap();
        write!(f, "Commitment({})", hex::encode(&bytes[3 * 8 + 4..]))
    }
}

impl From<Commitment> for poseidon377::Fq {
    fn from(commitment: Commitment) -> Self {
        commitment.0
    }
}

impl From<poseidon377::Fq> for Commitment {
    fn from(commitment: poseidon377::Fq) -> Self {
        Commitment(commitment)
    }
}

mod eternity;
pub use eternity::{
    epoch::{block::Block, Epoch},
    error, Eternity, Position, Proof, Root,
};

pub mod epoch {
    //! [`Epoch`]s within [`Eternity`](super::Eternity)s, and their [`Root`]s and [`Proof`]s of inclusion.
    pub use crate::eternity::epoch::*;
}

pub mod block {
    //! [`Block`]s within [`Epoch`](super::Epoch)s, and their [`Root`]s and [`Proof`]s of inclusion.
    pub use crate::eternity::epoch::block::*;
}

/// When inserting a [`Commitment`] into an [`Eternity`], [`Epoch`], or [`Block`], should we
/// [`Keep`] it to allow it to be witnessed later, or [`Forget`] about it after updating the root
/// hash?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub enum Witness {
    /// Keep this commitment so it can be witnessed later.
    Keep,
    /// Forget this commitment so it does not take up space, but it cannot be witnessed later.
    Forget,
}

/// When inserting a [`Commitment`] into an [`Eternity`], [`Epoch`], or [`Block`], this flag
/// indicates that we should immediately forget about it to save space, because we will not want to
/// witness its presence later.
pub use Witness::Forget;

/// When inserting a [`Commitment`] into an [`Eternity`], [`Epoch`], or [`Block`], this flag
/// indicates that we should keep this commitment to allow it to be witnessed later.
pub use Witness::Keep;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Eternity, [u8; 104]);
    }

    #[test]
    fn check_eternity_proof_size() {
        static_assertions::assert_eq_size!(Proof, [u8; 2344]);
    }
}

#[cfg(any(test, feature = "arbitrary"))]
pub mod arbitrary {
    //! Arbitrary implementation for [`Commitment`]s.

    use super::Commitment;

    impl proptest::arbitrary::Arbitrary for Commitment {
        type Parameters = Vec<Commitment>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            CommitmentStrategy(args)
        }

        type Strategy = CommitmentStrategy;
    }

    /// A [`proptest`] [`Strategy`](proptest::strategy::Strategy) for generating [`Commitment`]s.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub struct CommitmentStrategy(Vec<Commitment>);

    impl CommitmentStrategy {
        /// Create a new [`CommitmentStrategy`] that will generate arbitrary [`Commitment`]s.
        pub fn arbitrary() -> Self {
            Self::one_of(vec![])
        }

        /// Create a new [`CommitmentStrategy`] that will only produce the given [`Commitment`]s.
        ///
        /// If the given vector is empty, this will generate arbitrary commitments instead.
        pub fn one_of(commitments: Vec<Commitment>) -> Self {
            CommitmentStrategy(commitments)
        }
    }

    impl proptest::strategy::Strategy for CommitmentStrategy {
        type Tree = proptest::strategy::Just<Commitment>;

        type Value = Commitment;

        fn new_tree(
            &self,
            runner: &mut proptest::test_runner::TestRunner,
        ) -> proptest::strategy::NewTree<Self> {
            use proptest::prelude::{Rng, RngCore};
            let rng = runner.rng();
            if !self.0.is_empty() {
                Ok(proptest::strategy::Just(
                    *rng.sample(rand::distributions::Slice::new(&self.0).unwrap()),
                ))
            } else {
                let parts = [
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                ];
                Ok(proptest::strategy::Just(Commitment(decaf377::Fq::new(
                    ark_ff::BigInteger256(parts),
                ))))
            }
        }
    }
}
