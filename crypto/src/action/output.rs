use ark_ff::UniformRand;
use bytes::Bytes;
use rand_core::{CryptoRng, RngCore};
use std::convert::{TryFrom, TryInto};

use penumbra_proto::{transaction, Protobuf};

use super::{
    constants::{NOTE_ENCRYPTION_BYTES, OVK_WRAPPED_LEN_BYTES},
    error::ProtoError,
};
use crate::{
    ka, memo::MemoCiphertext, note, proofs::transparent::OutputProof, value, Address, Fq, Fr, Note,
    Value,
};

pub struct Output {
    pub body: Body,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES],
}

impl Protobuf<transaction::Output> for Output {}

impl From<Output> for transaction::Output {
    fn from(msg: Output) -> Self {
        transaction::Output {
            body: Some(msg.body.into()),
            encrypted_memo: Bytes::copy_from_slice(&msg.encrypted_memo.0),
            ovk_wrapped_key: Bytes::copy_from_slice(&msg.ovk_wrapped_key),
        }
    }
}

impl TryFrom<transaction::Output> for Output {
    type Error = ProtoError;

    fn try_from(proto: transaction::Output) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or(ProtoError::OutputBodyMalformed)?
            .try_into()
            .map_err(|_| ProtoError::OutputBodyMalformed)?;

        let encrypted_memo = MemoCiphertext(
            proto.encrypted_memo[..]
                .try_into()
                .map_err(|_| ProtoError::OutputMalformed)?,
        );

        let ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES] = proto.ovk_wrapped_key[..]
            .try_into()
            .map_err(|_| ProtoError::OutputMalformed)?;

        Ok(Output {
            body,
            encrypted_memo,
            ovk_wrapped_key,
        })
    }
}

pub struct Body {
    pub value_commitment: value::Commitment,
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; NOTE_ENCRYPTION_BYTES],
    pub proof: OutputProof,
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        v_blinding: Fr,
        dest: &Address,
    ) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let value_commitment = value.commit(v_blinding);

        let note_blinding = Fq::rand(rng);

        let note = Note::new(
            dest.diversified_generator(),
            dest.transmission_key(),
            value,
            note_blinding,
        );
        let note_commitment = note.commit().expect("transmission key is valid");
        // TODO: Encrypt note here and add to a field in the Body struct (later).
        // TEMP
        let encrypted_note = [0u8; NOTE_ENCRYPTION_BYTES];

        let esk = ka::Secret::new(rng);
        let ephemeral_key = esk.diversified_public(&note.diversified_generator);

        let proof = OutputProof {
            g_d: *dest.diversified_generator(),
            pk_d: *dest.transmission_key(),
            value,
            v_blinding,
            note_blinding,
            esk,
        };

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
            encrypted_note,
            proof,
        }
    }
}

impl Protobuf<transaction::OutputBody> for Body {}

impl From<Body> for transaction::OutputBody {
    fn from(msg: Body) -> Self {
        let cv_bytes: [u8; 32] = msg.value_commitment.into();
        let cm_bytes: [u8; 32] = msg.note_commitment.into();
        let proof: Vec<u8> = msg.proof.into();
        transaction::OutputBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            cm: Bytes::copy_from_slice(&cm_bytes),
            ephemeral_key: Bytes::copy_from_slice(&msg.ephemeral_key.0),
            encrypted_note: Bytes::copy_from_slice(&msg.encrypted_note),
            zkproof: Bytes::copy_from_slice(&proof[..]),
        }
    }
}

impl TryFrom<transaction::OutputBody> for Body {
    type Error = ProtoError;

    fn try_from(proto: transaction::OutputBody) -> anyhow::Result<Self, Self::Error> {
        Ok(Body {
            value_commitment: (proto.cv[..])
                .try_into()
                .map_err(|_| ProtoError::OutputBodyMalformed)?,
            note_commitment: (proto.cm[..])
                .try_into()
                .map_err(|_| ProtoError::OutputBodyMalformed)?,
            ephemeral_key: ka::Public::try_from(&proto.ephemeral_key[..])
                .map_err(|_| ProtoError::OutputBodyMalformed)?,
            encrypted_note: proto.encrypted_note[..]
                .try_into()
                .map_err(|_| ProtoError::OutputBodyMalformed)?,
            proof: proto.zkproof[..]
                .try_into()
                .map_err(|_| ProtoError::OutputBodyMalformed)?,
        })
    }
}
