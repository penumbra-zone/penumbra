use decaf377_rdsa::{Signature, SpendAuth};

use penumbra_proto::{
    core::{component::sct::v1::commitment_source::Transaction, transaction::v1 as pb},
    DomainType,
};
use penumbra_txhash::EffectHash;

/// Authorization data returned in response to some signing request, which may be a request to
/// authorize a transaction, a validator definition, or a validator vote.
#[derive(Clone, Debug)]
pub enum AuthorizationData {
    /// Authorization data for a transaction.
    Transaction(TransactionAuthorizationData),
    /// Authorization signature for a validator definition.
    ValidatorDefinition(Signature<SpendAuth>),
    /// Authorization signature for a validator vote.
    ValidatorVote(Signature<SpendAuth>),
}

/// Authorization data returned in response to a
/// [`TransactionDescription`](crate::TransactionDescription).
#[derive(Clone, Debug)]
pub struct TransactionAuthorizationData {
    /// The computed authorization hash for the approved transaction.
    pub effect_hash: Option<EffectHash>,
    /// The required spend authorization signatures, returned in the same order as the Spend actions
    /// in the original request.
    pub spend_auths: Vec<Signature<SpendAuth>>,
    /// The required delegator vote authorization signatures, returned in the same order as the
    /// DelegatorVote actions in the original request.
    pub delegator_vote_auths: Vec<Signature<SpendAuth>>,
}

impl DomainType for AuthorizationData {
    type Proto = pb::AuthorizationData;
}

impl From<AuthorizationData> for pb::AuthorizationData {
    fn from(msg: AuthorizationData) -> Self {
        match msg {
            AuthorizationData::Transaction(TransactionAuthorizationData {
                effect_hash,
                spend_auths,
                delegator_vote_auths,
            }) => Self {
                effect_hash: effect_hash.map(Into::into),
                spend_auths: spend_auths.into_iter().map(Into::into).collect(),
                delegator_vote_auths: delegator_vote_auths.into_iter().map(Into::into).collect(),
                validator_definition_auth: None,
                validator_vote_auth: None,
            },
            AuthorizationData::ValidatorDefinition(sig) => Self {
                effect_hash: None,
                spend_auths: vec![],
                delegator_vote_auths: vec![],
                validator_definition_auth: Some(sig.into()),
                validator_vote_auth: None,
            },
            AuthorizationData::ValidatorVote(sig) => Self {
                effect_hash: None,
                spend_auths: vec![],
                delegator_vote_auths: vec![],
                validator_definition_auth: None,
                validator_vote_auth: Some(sig.into()),
            },
        }
    }
}

impl TryFrom<pb::AuthorizationData> for AuthorizationData {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizationData) -> Result<Self, Self::Error> {
        if let Some(sig) = value.validator_definition_auth {
            if value.effect_hash.is_some()
                || !value.spend_auths.is_empty()
                || !value.delegator_vote_auths.is_empty()
            {
                anyhow::bail!("unexpected fields in validator definition authorization");
            }
            Ok(Self::ValidatorDefinition(TryInto::try_into(sig)?))
        } else if let Some(sig) = value.validator_vote_auth {
            if value.effect_hash.is_some()
                || !value.spend_auths.is_empty()
                || !value.delegator_vote_auths.is_empty()
            {
                anyhow::bail!("unexpected fields in validator vote authorization");
            }
            Ok(Self::ValidatorVote(TryInto::try_into(sig)?))
        } else {
            Ok(Self::Transaction(TransactionAuthorizationData {
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
            }))
        }
    }
}
