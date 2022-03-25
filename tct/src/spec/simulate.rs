use std::fmt::Debug;

use crate::{self as real, internal::hash::Hash, spec, Commitment, Witness};

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

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Action {
        Insert(Witness, Commitment),
        Forget(Commitment),
        InsertEpoch(Vec<super::epoch::Action>),
        InsertEpochRoot(Hash),
        InsertBlock(Vec<super::block::Action>),
        InsertBlockRoot(Hash),
    }
}

pub mod epoch {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Action {
        Insert(Witness, Commitment),
        Forget(Commitment),
        InsertBlock(Vec<super::block::Action>),
        InsertBlockRoot(Hash),
    }
}

pub mod block {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Action {
        Insert(Witness, Commitment),
        Forget(Commitment),
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
