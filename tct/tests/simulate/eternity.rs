use proptest::prelude::*;

use super::*;

use real::arbitrary::CommitmentStrategy;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Params"))]
pub enum Action {
    Insert(
        Witness,
        #[proptest(strategy = "CommitmentStrategy::one_of(params.commitments.clone())")] Commitment,
    ),
    Forget(
        #[proptest(strategy = "CommitmentStrategy::one_of(params.commitments.clone())")] Commitment,
    ),
    InsertEpoch(
        #[proptest(
            strategy = "prop::collection::vec(any_with::<epoch::Action>(params.clone()), 0..params.max_tier_actions)"
        )]
        Vec<epoch::Action>,
    ),
    InsertEpochRoot(real::epoch::Root),
    InsertBlock(
        #[proptest(
            strategy = "prop::collection::vec(any_with::<block::Action>(params.commitments), 0..params.max_tier_actions)"
        )]
        Vec<block::Action>,
    ),
    InsertBlockRoot(real::block::Root),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
pub enum Observation {
    Witness(#[proptest(strategy = "CommitmentStrategy::one_of(params)")] Commitment),
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
