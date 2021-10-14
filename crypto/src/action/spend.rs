use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use crate::Fr;

use crate::{
    keys::SpendKey,
    merkle::proof::MerklePath,
    proofs::transparent::SpendProof,
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
        spend_key: SpendKey,
        _merkle_path: MerklePath,
        note: Note,
    ) -> Self {
        // TODO: Derive nullifier from note commitment, note position, and
        // nullifier deriving key
        // See p.55 ZCash spec
        let nullifier = Nullifier::new();

        let v_blinding = Fr::rand(&mut rng);
        let value_commitment = note.value.commit(v_blinding);

        let body = Body::new(
            &mut rng,
            value_commitment,
            nullifier,
            *spend_key.spend_auth_key(),
        );

        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let auth_sig = rsk.sign(rng, &body.serialize());

        Spend { body, auth_sig }
    }
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
        mut rng: R,
        value_commitment: value::Commitment,
        nullifier: Nullifier,
        ask: SigningKey<SpendAuth>,
    ) -> Body {
        let a = Fr::rand(&mut rng);
        let rk = ask.randomize(&a).into();
        let proof = SpendProof::new();
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
