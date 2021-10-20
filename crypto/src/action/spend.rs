use std::convert::{TryFrom, TryInto};

use ark_ff::UniformRand;
use bytes::Bytes;
use rand_core::{CryptoRng, RngCore};

use penumbra_proto::{transaction, Protobuf};

use crate::{
    action::error::ProtoError,
    merkle,
    proofs::transparent::{SpendProof, SPEND_PROOF_LEN_BYTES},
    rdsa::{Signature, SigningKey, SpendAuth, VerificationKey},
    value, Fr, Nullifier,
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

impl Protobuf<transaction::SpendBody> for Body {}

impl From<Body> for transaction::SpendBody {
    fn from(msg: Body) -> Self {
        let cv_bytes: [u8; 32] = msg.value_commitment.into();
        let nullifier_bytes: [u8; 32] = msg.nullifier.into();
        let rk_bytes: [u8; 32] = msg.rk.into();
        let proof_bytes: [u8; SPEND_PROOF_LEN_BYTES] = msg.proof.into();
        transaction::SpendBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            nullifier: Bytes::copy_from_slice(&nullifier_bytes),
            rk: Bytes::copy_from_slice(&rk_bytes),
            zkproof: Bytes::copy_from_slice(&proof_bytes),
        }
    }
}

impl TryFrom<transaction::SpendBody> for Body {
    type Error = ProtoError;

    fn try_from(proto: transaction::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let value_commitment: value::Commitment = (proto.cv[..])
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        let nullifier = (proto.nullifier[..])
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        let rk_bytes: [u8; 32] = (proto.rk[..])
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;
        let rk = rk_bytes
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        let proof = (proto.zkproof[..])
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        Ok(Body {
            value_commitment,
            nullifier,
            rk,
            proof,
        })
    }
}
