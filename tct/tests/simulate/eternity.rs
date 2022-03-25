use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Action {
    Insert(Witness, Commitment),
    Forget(Commitment),
    InsertEpoch(Vec<epoch::Action>),
    InsertEpochRoot(real::epoch::Root),
    InsertBlock(Vec<block::Action>),
    InsertBlockRoot(real::block::Root),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub enum Observation {
    Witness(Commitment),
    Root,
    CurrentEpochRoot,
    CurrentBlockRoot,
    Position,
    WitnessedCount,
    IsEmpty,
}

impl Simulate for Action {
    type Spec = spec::eternity::Builder;
    type Real = real::Eternity;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        match self {
            Action::Insert(witness, commitment) => assert_eq!(
                spec.insert(witness, commitment),
                real.insert(witness, commitment).map_err(Into::into)
            ),
            Action::Forget(commitment) => {
                assert_eq!(spec.forget(commitment), real.forget(commitment))
            }
            Action::InsertEpoch(actions) => {
                let mut spec_epoch = spec::epoch::Builder::default();
                let mut real_epoch = real::Epoch::default();
                actions.simulate(&mut spec_epoch, &mut real_epoch);
                assert_eq!(
                    spec.insert_epoch(spec_epoch),
                    real.insert_epoch(real_epoch).map_err(Into::into)
                );
            }
            Action::InsertEpochRoot(root) => {
                assert_eq!(
                    spec.insert_epoch_root(root),
                    real.insert_epoch_root(root).map_err(Into::into)
                )
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
