use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use bytes::Bytes;
use penumbra_crypto::{
    balance,
    proofs::groth16::SpendProof,
    rdsa::{Signature, SpendAuth, VerificationKey},
    Nullifier,
};
use penumbra_proto::{core::transaction::v1alpha1 as transaction, DomainType};

use crate::{view::action_view::SpendView, ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Clone, Debug)]
pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: SpendProof,
}

impl IsAction for Spend {
    fn balance_commitment(&self) -> balance::Commitment {
        self.body.balance_commitment
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let spend_view = match txp.spend_nullifiers.get(&self.body.nullifier) {
            Some(note) => SpendView::Visible {
                spend: self.to_owned(),
                note: txp.view_note(note.to_owned()),
            },
            None => SpendView::Opaque {
                spend: self.to_owned(),
            },
        };

        ActionView::Spend(spend_view)
    }
}

impl DomainType for Spend {
    type Proto = transaction::Spend;
}

impl From<Spend> for transaction::Spend {
    fn from(msg: Spend) -> Self {
        transaction::Spend {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
            proof: Some(msg.proof.into()),
        }
    }
}

impl TryFrom<transaction::Spend> for Spend {
    type Error = Error;

    fn try_from(proto: transaction::Spend) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or_else(|| anyhow::anyhow!("missing spend body"))?
            .try_into()
            .context("malformed spend body")?;
        let auth_sig = proto
            .auth_sig
            .ok_or_else(|| anyhow::anyhow!("missing auth sig"))?
            .try_into()
            .context("malformed auth sig")?;
        let proof = proto
            .proof
            .ok_or_else(|| anyhow::anyhow!("missing proof"))?
            .try_into()
            .context("malformed spend proof")?;

        Ok(Spend {
            body,
            auth_sig,
            proof,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub balance_commitment: balance::Commitment,
    pub nullifier: Nullifier,
    pub rk: VerificationKey<SpendAuth>,
}

impl DomainType for Body {
    type Proto = transaction::SpendBody;
}

impl From<Body> for transaction::SpendBody {
    fn from(msg: Body) -> Self {
        let nullifier_bytes: [u8; 32] = msg.nullifier.into();
        let rk_bytes: [u8; 32] = msg.rk.into();
        transaction::SpendBody {
            balance_commitment: Some(msg.balance_commitment.into()),
            nullifier: Bytes::copy_from_slice(&nullifier_bytes),
            rk: Bytes::copy_from_slice(&rk_bytes),
        }
    }
}

impl TryFrom<transaction::SpendBody> for Body {
    type Error = Error;

    fn try_from(proto: transaction::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let balance_commitment: balance::Commitment = proto
            .balance_commitment
            .ok_or_else(|| anyhow::anyhow!("missing value commitment"))?
            .try_into()
            .context("malformed balance commitment")?;

        let nullifier = (proto.nullifier[..])
            .try_into()
            .context("malformed nullifier")?;

        let rk_bytes: [u8; 32] = (proto.rk[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected 32-byte rk"))?;
        let rk = rk_bytes.try_into().context("malformed rk")?;

        Ok(Body {
            balance_commitment,
            nullifier,
            rk,
        })
    }
}
