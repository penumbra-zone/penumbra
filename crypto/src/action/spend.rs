use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use crate::Fr;

use crate::{
    merkle,
    proofs::transparent::SpendProof,
    rdsa::{Signature, SigningKey, SpendAuth, VerificationKey},
    value, Nullifier,
};

pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
}

pub struct Body {
    pub value_commitment: value::Commitment,
    pub nullifier: Nullifier,
    // Randomized verification key.
    pub rk: VerificationKey<SpendAuth>,
    pub proof: SpendProof,
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value_commitment: value::Commitment,
        nullifier: Nullifier,
        ask: SigningKey<SpendAuth>,
        spend_auth_randomizer: Fr,
        merkle_path: merkle::Path,
    ) -> Body {
        let a = Fr::rand(rng);
        let rk = ask.randomize(&a).into();
        let proof = SpendProof {
            spend_auth_randomizer,
            merkle_path,
        };
        Body {
            value_commitment,
            nullifier,
            rk,
            proof,
        }
    }

    // xx Replace with proto serialization into `SpendBody`?
    pub fn serialize(&self) -> &[u8] {
        todo!();
    }
}
