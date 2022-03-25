use std::fmt::Debug;

use proptest_derive::Arbitrary;

use penumbra_tct::{self as real, spec, Commitment, Witness};

/// Simulate an action being run on both a specification and a real implementation simultaneously.
pub trait Simulate {
    /// The specification type for the simulation.
    type Spec;

    /// The real type which is being simulated.
    type Real;

    /// Run the same operation `self` on both `spec` and `real`.
    ///
    /// # Panics
    ///
    /// Should panic if the result of the operation differs between the two.
    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real);
}

impl<I: IntoIterator<Item = T>, T: Simulate> Simulate for I {
    type Spec = T::Spec;
    type Real = T::Real;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        for action in self.into_iter() {
            action.simulate(spec, real);
        }
    }
}

pub mod eternity {
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
}

pub mod epoch {
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
}

pub mod block {
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
}
