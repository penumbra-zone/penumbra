use std::convert::{TryFrom, TryInto};

use bytes::Bytes;
use penumbra_crypto::{ka, memo::MemoCiphertext, note, value, Fr, Note};
use penumbra_proto::{transaction, Protobuf};

use anyhow::Error;

#[derive(Clone, Debug)]
pub struct Output {
    pub body: Body,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; note::OVK_WRAPPED_LEN_BYTES],
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
    type Error = Error;

    fn try_from(proto: transaction::Output) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or(anyhow::anyhow!("output body malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("output body malformed"))?;

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
        })
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub value_commitment: value::Commitment,
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
}

impl Body {
    pub fn new(note: Note, v_blinding: Fr, esk: &ka::Secret) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.

        // Outputs subtract from the transaction value balance, so commit to -value.
        let value_commitment = -note.value().commit(v_blinding);
        let note_commitment = note.commit();

        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(esk);

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
            encrypted_note,
        }
    }
}

impl Protobuf<transaction::OutputBody> for Body {}

impl From<Body> for transaction::OutputBody {
    fn from(msg: Body) -> Self {
        let cv_bytes: [u8; 32] = msg.value_commitment.into();
        let cm_bytes: [u8; 32] = msg.note_commitment.into();
        transaction::OutputBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            cm: Bytes::copy_from_slice(&cm_bytes),
            ephemeral_key: Bytes::copy_from_slice(&msg.ephemeral_key.0),
            encrypted_note: Bytes::copy_from_slice(&msg.encrypted_note),
        }
    }
}

impl TryFrom<transaction::OutputBody> for Body {
    type Error = Error;

    fn try_from(proto: transaction::OutputBody) -> anyhow::Result<Self, Self::Error> {
        Ok(Body {
            value_commitment: (proto.cv[..])
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            note_commitment: (proto.cm[..])
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            ephemeral_key: ka::Public::try_from(&proto.ephemeral_key[..])
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            encrypted_note: proto.encrypted_note[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}
