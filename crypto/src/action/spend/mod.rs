use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::Fr;

use crate::{
    rdsa::{SigningKey, SpendAuth, VerificationKey},
    value, Nullifier,
};

pub struct Body {
    pub value_commitment: value::Commitment,
    pub nullifier: Nullifier,
    // Randomized verification key.
    pub rk: VerificationKey<SpendAuth>,
    // TODO: Proof
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        mut rng: R,
        value_commitment: value::Commitment,
        nullifier: Nullifier,
        ask: SigningKey<SpendAuth>,
    ) -> Body {
        let a = Fr::rand(&mut rng);
        let rk = ask.randomize(&a).into();
        Body {
            value_commitment,
            nullifier,
            rk,
        }
    }
}
