use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use bytes::Bytes;
use penumbra_crypto::{
    memo::MemoCiphertext, note, proofs::transparent::OutputProof, value, NotePayload,
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
            .ok_or_else(|| anyhow::anyhow!("missing output body"))?
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
