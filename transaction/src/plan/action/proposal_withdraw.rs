use decaf377::{FieldExt, Fr};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::{ProposalWithdraw, ProposalWithdrawBody};

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ProposalWithdrawPlan",
    into = "pb::ProposalWithdrawPlan"
)]
pub struct ProposalWithdrawPlan {
    /// The body of the proposal withdrawal.
    pub body: ProposalWithdrawBody,
    /// The randomizer to use for the signature.
    pub randomizer: Fr,
}

impl ProposalWithdrawPlan {
    /// Create a proposal withdraw action from a plan, given the fvk and the authorization signature.
    pub fn withdraw(self, auth_sig: Signature<SpendAuth>) -> ProposalWithdraw {
        ProposalWithdraw {
            body: self.body,
            auth_sig,
        }
    }
}

impl Protobuf<pb::ProposalWithdrawPlan> for ProposalWithdrawPlan {}

impl From<ProposalWithdrawPlan> for pb::ProposalWithdrawPlan {
    fn from(inner: ProposalWithdrawPlan) -> Self {
        pb::ProposalWithdrawPlan {
            body: Some(inner.body.into()),
            randomizer: inner.randomizer.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::ProposalWithdrawPlan> for ProposalWithdrawPlan {
    type Error = anyhow::Error;

    fn try_from(value: pb::ProposalWithdrawPlan) -> Result<Self, Self::Error> {
        Ok(ProposalWithdrawPlan {
            body: value
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body in `ProposalWithdrawPlan`"))?
                .try_into()?,
            randomizer: Fr::from_bytes(value.randomizer.as_ref().try_into()?)?,
        })
    }
}
