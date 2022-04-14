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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub commitments: Vec<Commitment>,
    pub max_tier_actions: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            commitments: vec![],
            max_tier_actions: 100,
        }
    }
}

pub mod block;
pub mod epoch;
pub mod eternity;
