use std::convert::{TryFrom, TryInto};

use ark_ff::UniformRand;
use bytes::Bytes;
use rand_core::{CryptoRng, RngCore};

use penumbra_proto::{transaction, Protobuf};

use crate::{
    action::error::ProtoError,
    keys, merkle,
    proofs::transparent::SpendProof,
    rdsa::{Signature, SigningKey, SpendAuth, VerificationKey},
    value, Fr, Note, Nullifier,
};

pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
}

impl Protobuf<transaction::Spend> for Spend {}

impl From<Spend> for transaction::Spend {
    fn from(msg: Spend) -> Self {
        let sig_bytes: [u8; 64] = msg.auth_sig.into();
        transaction::Spend {
            body: Some(msg.body.into()),
            auth_sig: Bytes::copy_from_slice(&sig_bytes),
        }
    }
}

impl TryFrom<transaction::Spend> for Spend {
    type Error = ProtoError;

    fn try_from(proto: transaction::Spend) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or(ProtoError::SpendBodyMalformed)?
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        let sig_bytes: [u8; 64] = proto.auth_sig[..]
            .try_into()
            .map_err(|_| ProtoError::SpendBodyMalformed)?;

        Ok(Spend {
            body,
            auth_sig: sig_bytes.into(),
        })
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
        rng: &mut R,
        value_commitment: value::Commitment,
        ask: SigningKey<SpendAuth>,
        spend_auth_randomizer: Fr,
        merkle_path: merkle::Path,
        position: merkle::Position,
        note: Note,
        v_blinding: Fr,
        nk: keys::NullifierKey,
    ) -> Body {
        let a = Fr::rand(rng);
        let rk = ask.randomize(&a).into();
        let note_commitment = note.commit().expect("transmission key is valid");
        let proof = SpendProof {
            merkle_path,
            position,
            g_d: note.diversified_generator,
            pk_d: note.transmission_key,
            value: note.value,
            v_blinding,
            note_commitment,
            note_blinding: note.note_blinding,
            spend_auth_randomizer,
            ak: ask.into(),
            nk,
        };
        Body {
            value_commitment,
            nullifier: nk.nf(position, &note_commitment),
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
        let proof: Vec<u8> = msg.proof.into();
        transaction::SpendBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            nullifier: Bytes::copy_from_slice(&nullifier_bytes),
            rk: Bytes::copy_from_slice(&rk_bytes),
            zkproof: Bytes::copy_from_slice(&proof[..]),
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
