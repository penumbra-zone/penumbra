//! Generating random values of various types.

use decaf377::Fq;
use rand::{distributions::Distribution, Rng};

use super::StateCommitment;
use crate::{
    builder::{block, epoch},
    structure::Hash,
    Root,
};

struct UniformFq;

impl Distribution<Fq> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Fq {
        let mut bytes = [0u8; 32];
        loop {
            rng.fill_bytes(&mut bytes);
            if let Ok(fq) = Fq::from_bytes_checked(&bytes) {
                return fq;
            }
        }
    }
}

impl Distribution<StateCommitment> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StateCommitment {
        StateCommitment(self.sample(rng))
    }
}

impl StateCommitment {
    /// Generate a random [`Commitment`].
    pub fn random(mut rng: impl Rng) -> Self {
        rng.sample(UniformFq)
    }
}

impl Distribution<Hash> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
        Hash::new(self.sample(rng))
    }
}

impl Hash {
    /// Generate a random [`struct@Hash`].
    pub fn random(mut rng: impl Rng) -> Self {
        rng.sample(UniformFq)
    }
}

impl Distribution<Root> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Root {
        Root(self.sample(rng))
    }
}

impl Root {
    /// Generate a random [`Root`].
    pub fn random(mut rng: impl Rng) -> Self {
        rng.sample(UniformFq)
    }
}

impl Distribution<epoch::Root> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> epoch::Root {
        epoch::Root(self.sample(rng))
    }
}

impl epoch::Root {
    /// Generate a random [`epoch::Root`].
    pub fn random(mut rng: impl Rng) -> Self {
        rng.sample(UniformFq)
    }
}

impl Distribution<block::Root> for UniformFq {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> block::Root {
        block::Root(self.sample(rng))
    }
}

impl block::Root {
    /// Generate a random [`block::Root`].
    pub fn random(mut rng: impl Rng) -> Self {
        rng.sample(UniformFq)
    }
}
