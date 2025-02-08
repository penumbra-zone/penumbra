use decaf377_rdsa::{Signature, SpendAuth};

use penumbra_sdk_proto::{core::transaction::v1 as pb, DomainType};
use penumbra_sdk_txhash::EffectHash;

/// Authorization data returned in response to a
/// [`TransactionDescription`](crate::TransactionDescription).
#[derive(Clone, Debug, Default)]
pub struct AuthorizationData {
    /// The computed authorization hash for the approved transaction.
    pub effect_hash: Option<EffectHash>,
    /// The required spend authorization signatures, returned in the same order as the Spend actions
    /// in the original request.
    pub spend_auths: Vec<Signature<SpendAuth>>,
    /// The required delegator vote authorization signatures, returned in the same order as the
    /// DelegatorVote actions in the original request.
    pub delegator_vote_auths: Vec<Signature<SpendAuth>>,
    /// The required LQT vote authorization signatures, returned in the same order as the
    /// actions in the original request
    pub lqt_vote_auths: Vec<Signature<SpendAuth>>,
}

impl DomainType for AuthorizationData {
    type Proto = pb::AuthorizationData;
}

impl From<AuthorizationData> for pb::AuthorizationData {
    fn from(msg: AuthorizationData) -> Self {
        Self {
            effect_hash: msg.effect_hash.map(Into::into),
            spend_auths: msg.spend_auths.into_iter().map(Into::into).collect(),
            delegator_vote_auths: msg
                .delegator_vote_auths
                .into_iter()
                .map(Into::into)
                .collect(),
            lqt_vote_auths: msg.lqt_vote_auths.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::AuthorizationData> for AuthorizationData {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizationData) -> Result<Self, Self::Error> {
        Ok(Self {
            effect_hash: value.effect_hash.map(TryInto::try_into).transpose()?,
            spend_auths: value
                .spend_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            delegator_vote_auths: value
                .delegator_vote_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            lqt_vote_auths: value
                .lqt_vote_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
