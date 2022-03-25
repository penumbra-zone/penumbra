use real::arbitrary::CommitmentStrategy;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
pub enum Action {
    Insert(
        Witness,
        #[proptest(strategy = "CommitmentStrategy::one_of(params.clone())")] Commitment,
    ),
    Forget(#[proptest(strategy = "CommitmentStrategy::one_of(params)")] Commitment),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
pub enum Observation {
    Witness(#[proptest(strategy = "CommitmentStrategy::one_of(params)")] Commitment),
    Root,
    Position,
    WitnessedCount,
    IsEmpty,
}

impl Simulate for Action {
    type Spec = spec::block::Builder;
    type Real = real::Block;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        match self {
            Action::Insert(witness, commitment) => assert_eq!(
                spec.insert(witness, commitment),
                real.insert(witness, commitment).map_err(Into::into)
            ),
            Action::Forget(commitment) => {
                assert_eq!(spec.forget(commitment), real.forget(commitment))
            }
        }
    }
}
