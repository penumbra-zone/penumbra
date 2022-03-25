use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Action {
    Insert(Witness, Commitment),
    Forget(Commitment),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Observation {
    Witness(Commitment),
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
