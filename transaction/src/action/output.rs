use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use bytes::Bytes;
use penumbra_crypto::{
    ka,
    keys::OutgoingViewingKey,
    memo::{MemoCiphertext, MemoPlaintext},
    note,
    proofs::transparent::OutputProof,
    value, Address, Fr, Note,
};
use penumbra_proto::{transaction as pb, Protobuf};
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct Output {
    pub body: Body,
    pub value_commitment: value::Commitment,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; note::OVK_WRAPPED_LEN_BYTES],
    pub proof: OutputProof,
}

impl Output {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        note: Note,
        memo: MemoPlaintext,
        dest: &Address,
        ovk: &OutgoingViewingKey,
        v_blinding: Fr,
    ) -> Output {
        let esk = ka::Secret::new(rng);
        let diversified_generator = note.diversified_generator();
        let transmission_key = note.transmission_key();

        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.

        // Outputs subtract from the transaction value balance, so commit to -value.
        let value_commitment = -note.value().commit(v_blinding);
        let note_commitment = note.commit();

        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);
        let encrypted_memo = memo.encrypt(&esk, dest);
        let ovk_wrapped_key = note.encrypt_key(&esk, ovk, value_commitment);

        let proof = OutputProof {
            g_d: diversified_generator,
            pk_d: transmission_key,
            value: note.value(),
            v_blinding,
            note_blinding: note.note_blinding(),
            esk: esk.clone(),
        };

        Self {
            body: Body {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            value_commitment,
            encrypted_memo,
            ovk_wrapped_key,
            proof,
        }
    }
}

impl Protobuf<pb::Output> for Output {}

impl From<Output> for pb::Output {
    fn from(output: Output) -> Self {
        let cv_bytes: [u8; 32] = output.value_commitment.into();
        let proof: Vec<u8> = output.proof.into();
        pb::Output {
            body: Some(output.body.into()),
            cv: cv_bytes.to_vec().into(),
            encrypted_memo: Bytes::copy_from_slice(&output.encrypted_memo.0),
            ovk_wrapped_key: Bytes::copy_from_slice(&output.ovk_wrapped_key),
            zkproof: proof.into(),
        }
    }
}

impl TryFrom<pb::Output> for Output {
    type Error = Error;

    fn try_from(proto: pb::Output) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or(anyhow::anyhow!("missing output body"))?
            .try_into()
            .map_err(|e: Error| e.context("output body malformed"))?;

        let encrypted_memo = MemoCiphertext(
            proto.encrypted_memo[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output malformed"))?,
        );

        let ovk_wrapped_key: [u8; note::OVK_WRAPPED_LEN_BYTES] = proto.ovk_wrapped_key[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("output malformed"))?;

        Ok(Output {
            body,
            encrypted_memo,
            ovk_wrapped_key,
            value_commitment: (proto.cv[..])
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            proof: proto.zkproof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
}

impl Body {}

impl Protobuf<pb::OutputBody> for Body {}

impl From<Body> for pb::OutputBody {
    fn from(msg: Body) -> Self {
        pb::OutputBody {
            note_commitment: Some(msg.note_commitment.into()),
            ephemeral_key: Bytes::copy_from_slice(&msg.ephemeral_key.0),
            encrypted_note: Bytes::copy_from_slice(&msg.encrypted_note),
        }
    }
}

impl TryFrom<pb::OutputBody> for Body {
    type Error = Error;

    fn try_from(proto: pb::OutputBody) -> anyhow::Result<Self, Self::Error> {
        Ok(Body {
            note_commitment: proto
                .note_commitment
                .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
                .try_into()?,
            ephemeral_key: ka::Public::try_from(&proto.ephemeral_key[..])
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            encrypted_note: proto.encrypted_note[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}
