use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use bytes::Bytes;
use penumbra_crypto::{
    ka,
    keys::OutgoingViewingKey,
    memo::{MemoCiphertext, MemoPlaintext},
    note,
    proofs::transparent::OutputProof,
    value, Address, Fr, Note, NotePayload,
};
use penumbra_proto::{transaction as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct Output {
    pub body: Body,
    pub proof: OutputProof,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub note_payload: NotePayload,
    pub value_commitment: value::Commitment,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; note::OVK_WRAPPED_LEN_BYTES],
}

impl Output {
    pub fn new(
        esk: ka::Secret,
        note: Note,
        memo: MemoPlaintext,
        dest: &Address,
        ovk: &OutgoingViewingKey,
        v_blinding: Fr,
    ) -> Output {
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
                note_payload: NotePayload {
                    note_commitment,
                    ephemeral_key,
                    encrypted_note,
                },
                value_commitment,
                encrypted_memo,
                ovk_wrapped_key,
            },
            proof,
        }
    }
}

impl Protobuf<pb::Output> for Output {}

impl From<Output> for pb::Output {
    fn from(output: Output) -> Self {
        let proof: Vec<u8> = output.proof.into();
        pb::Output {
            body: Some(output.body.into()),
            zkproof: proof.into(),
        }
    }
}

impl TryFrom<pb::Output> for Output {
    type Error = Error;

    fn try_from(proto: pb::Output) -> anyhow::Result<Self, Self::Error> {
        Ok(Output {
            body: proto
                .body
                .ok_or_else(|| anyhow::anyhow!("missing output body"))?
                .try_into()?,
            proof: proto.zkproof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}

impl Protobuf<pb::OutputBody> for Body {}

impl From<Body> for pb::OutputBody {
    fn from(output: Body) -> Self {
        let cv_bytes: [u8; 32] = output.value_commitment.into();
        pb::OutputBody {
            note_payload: Some(output.note_payload.into()),
            cv: cv_bytes.to_vec().into(),
            encrypted_memo: Bytes::copy_from_slice(&output.encrypted_memo.0),
            ovk_wrapped_key: Bytes::copy_from_slice(&output.ovk_wrapped_key),
        }
    }
}

impl TryFrom<pb::OutputBody> for Body {
    type Error = Error;

    fn try_from(proto: pb::OutputBody) -> anyhow::Result<Self, Self::Error> {
        let note_payload = proto
            .note_payload
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

        Ok(Body {
            note_payload,
            encrypted_memo,
            ovk_wrapped_key,
            value_commitment: (proto.cv[..])
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}
