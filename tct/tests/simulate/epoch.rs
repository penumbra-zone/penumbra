use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Action {
    Insert(Witness, Commitment),
    Forget(Commitment),
    InsertBlock(Vec<block::Action>),
    InsertBlockRoot(real::block::Root),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Observation {
    Witness(Commitment),
    Root,
    CurrentBlockRoot,
    Position,
    WitnessedCount,
    IsEmpty,
}

impl Simulate for Action {
    type Spec = spec::epoch::Builder;
    type Real = real::Epoch;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        match self {
            Action::Insert(witness, commitment) => assert_eq!(
                spec.insert(witness, commitment),
                real.insert(witness, commitment).map_err(Into::into)
            ),
            Action::Forget(commitment) => {
                assert_eq!(spec.forget(commitment), real.forget(commitment))
            }
            Action::InsertBlock(actions) => {
                let mut spec_block = spec::block::Builder::default();
                let mut real_block = real::Block::default();
                actions.simulate(&mut spec_block, &mut real_block);
                assert_eq!(
                    spec.insert_block(spec_block),
                    real.insert_block(real_block).map_err(Into::into)
                );
            }
            Action::InsertBlockRoot(root) => {
                assert_eq!(
                    spec.insert_block_root(root),
                    real.insert_block_root(root).map_err(Into::into)
                )
            }
        }
    }
}
