use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::Fr;

use crate::{
    keys::{Diversifier, SpendKey},
    merkle::proof::MerklePath,
    rdsa::{Signature, SigningKey, SpendAuth, VerificationKey},
    value, Note, Nullifier,
};

pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
}

impl Spend {
    pub fn new<R: RngCore + CryptoRng>(
        mut rng: R,
        diversifier: &Diversifier,
        spend_key: SpendKey,
        merkle_path: MerklePath,
        note: Note,
    ) -> Self {
        // Derive nullifier from MerklePath and key
        let nullifier = todo!();

        let value_commitment = todo!();

        let body = Body::new(
            &mut rng,
            value_commitment,
            nullifier,
            *spend_key.spend_auth_key(),
        );

        let auth_sig = todo!();

        Spend { body, auth_sig }
    }
}

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

    pub fn sign(&self) -> Signature<SpendAuth> {
        todo!()
    }
}
