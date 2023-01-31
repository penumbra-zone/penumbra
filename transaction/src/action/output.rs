use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use bytes::Bytes;
use penumbra_crypto::{
    balance,
    proofs::groth16::ZKOutputProof,
    symmetric::{OvkWrappedKey, WrappedMemoKey},
    EncryptedNote, Note,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};

use crate::{view::action_view::OutputView, ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Clone, Debug)]
pub struct Output {
    pub body: Body,
    pub proof: ZKOutputProof,
}

impl IsAction for Output {
    fn balance_commitment(&self) -> balance::Commitment {
        self.body.balance_commitment
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let note_commitment = self.body.note_payload.note_commitment;
        let epk = self.body.note_payload.ephemeral_key;
        // Retrieve payload key for associated note commitment
        let output_view = if let Some(payload_key) = txp.payload_keys.get(&note_commitment) {
            let decrypted_note = Note::decrypt_with_payload_key(
                &self.body.note_payload.encrypted_note,
                payload_key,
                &epk,
            );

            let decrypted_memo_key = self.body.wrapped_memo_key.decrypt_outgoing(payload_key);

            if let (Ok(decrypted_note), Ok(decrypted_memo_key)) =
                (decrypted_note, decrypted_memo_key)
            {
                // Neither decryption failed, so return the visible ActionView
                OutputView::Visible {
                    output: self.to_owned(),
                    note: decrypted_note,
                    payload_key: decrypted_memo_key,
                }
            } else {
                // One or both of the note or memo key is missing, so return the opaque ActionView
                OutputView::Opaque {
                    output: self.to_owned(),
                }
            }
        } else {
            // There was no payload key found, so return the opaque ActionView
            OutputView::Opaque {
                output: self.to_owned(),
            }
        };

        ActionView::Output(output_view)
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub note_payload: EncryptedNote,
    pub balance_commitment: balance::Commitment,
    pub ovk_wrapped_key: OvkWrappedKey,
    pub wrapped_memo_key: WrappedMemoKey,
}

impl DomainType for Output {
    type Proto = pb::Output;
}

impl From<Output> for pb::Output {
    fn from(output: Output) -> Self {
        let proof: Vec<u8> = output.proof.into();
        pb::Output {
            body: Some(output.body.into()),
            proof: proof.into(),
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
            proof: proto.proof[..]
                .try_into()
                .context("output proof malformed")?,
        })
    }
}

impl DomainType for Body {
    type Proto = pb::OutputBody;
}

impl From<Body> for pb::OutputBody {
    fn from(output: Body) -> Self {
        pb::OutputBody {
            note_payload: Some(output.note_payload.into()),
            balance_commitment: Some(output.balance_commitment.into()),
            wrapped_memo_key: Bytes::copy_from_slice(&output.wrapped_memo_key.0),
            ovk_wrapped_key: Bytes::copy_from_slice(&output.ovk_wrapped_key.0),
        }
    }
}

impl TryFrom<pb::OutputBody> for Body {
    type Error = Error;

    fn try_from(proto: pb::OutputBody) -> anyhow::Result<Self, Self::Error> {
        let note_payload = proto
            .note_payload
            .ok_or_else(|| anyhow::anyhow!("missing note payload"))?
            .try_into()
            .context("malformed note payload")?;

        let wrapped_memo_key = proto.wrapped_memo_key[..]
            .try_into()
            .context("malformed wrapped memo key")?;

        let ovk_wrapped_key: OvkWrappedKey = proto.ovk_wrapped_key[..]
            .try_into()
            .context("malformed ovk wrapped key")?;

        let balance_commitment = proto
            .balance_commitment
            .ok_or_else(|| anyhow::anyhow!("missing value commitment"))?
            .try_into()
            .context("malformed balance commitment")?;

        Ok(Body {
            note_payload,
            wrapped_memo_key,
            ovk_wrapped_key,
            balance_commitment,
        })
    }
}
