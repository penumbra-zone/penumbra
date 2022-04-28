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
