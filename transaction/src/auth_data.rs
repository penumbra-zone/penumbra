use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};

use crate::EffectHash;

/// Authorization data returned in response to a
/// [`TransactionDescription`](crate::TransactionDescription).
#[derive(Clone, Debug)]
pub struct AuthorizationData {
    /// The computed authorization hash for the approved transaction.
    pub effect_hash: EffectHash,
    /// The required spend authorization signatures, returned in the same order as the Spend actions
    /// in the original request.
    pub spend_auths: Vec<Signature<SpendAuth>>,
    /// The required withdraw proposal authorization signatures, returned in the same order as the
    /// ProposalWithdraw actions in the original request.
    pub withdraw_proposal_auths: Vec<Signature<SpendAuth>>,
}

impl Protobuf<pb::AuthorizationData> for AuthorizationData {}

impl From<AuthorizationData> for pb::AuthorizationData {
    fn from(msg: AuthorizationData) -> Self {
        Self {
            effect_hash: Some(msg.effect_hash.into()),
            spend_auths: msg.spend_auths.into_iter().map(Into::into).collect(),
            withdraw_proposal_auths: msg
                .withdraw_proposal_auths
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<pb::AuthorizationData> for AuthorizationData {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizationData) -> Result<Self, Self::Error> {
        Ok(Self {
            effect_hash: value
                .effect_hash
                .ok_or_else(|| anyhow::anyhow!("missing effect_hash"))?
                .try_into()?,
            spend_auths: value
                .spend_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            withdraw_proposal_auths: value
                .withdraw_proposal_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
