use penumbra_sdk_governance::ValidatorVoteBody;
use penumbra_sdk_proto::{custody::v1 as pb, DomainType};
use penumbra_sdk_stake::validator::Validator;
use penumbra_sdk_transaction::TransactionPlan;

use crate::PreAuthorization;

/// A transaction authorization request submitted to a custody service for approval.
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    /// The transaction plan to authorize.
    pub plan: TransactionPlan,
    /// Optionally, pre-authorization data, if required by the custodian.
    pub pre_authorizations: Vec<PreAuthorization>,
}

impl DomainType for AuthorizeRequest {
    type Proto = pb::AuthorizeRequest;
}

impl TryFrom<pb::AuthorizeRequest> for AuthorizeRequest {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizeRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plan: value
                .plan
                .ok_or_else(|| anyhow::anyhow!("missing plan"))?
                .try_into()?,
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<AuthorizeRequest> for pb::AuthorizeRequest {
    fn from(value: AuthorizeRequest) -> pb::AuthorizeRequest {
        Self {
            plan: Some(value.plan.into()),
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// A validator definition authorization request submitted to a custody service for approval.
#[derive(Debug, Clone)]
pub struct AuthorizeValidatorDefinitionRequest {
    /// The validator definition to authorize.
    pub validator_definition: Validator,
    /// Optionally, pre-authorization data, if required by the custodian.
    pub pre_authorizations: Vec<PreAuthorization>,
}

impl DomainType for AuthorizeValidatorDefinitionRequest {
    type Proto = pb::AuthorizeValidatorDefinitionRequest;
}

impl TryFrom<pb::AuthorizeValidatorDefinitionRequest> for AuthorizeValidatorDefinitionRequest {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizeValidatorDefinitionRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_definition: value
                .validator_definition
                .ok_or_else(|| anyhow::anyhow!("missing validator definition"))?
                .try_into()?,
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<AuthorizeValidatorDefinitionRequest> for pb::AuthorizeValidatorDefinitionRequest {
    fn from(value: AuthorizeValidatorDefinitionRequest) -> pb::AuthorizeValidatorDefinitionRequest {
        Self {
            validator_definition: Some(value.validator_definition.into()),
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// A validator vote authorization request submitted to a custody service for approval.
#[derive(Debug, Clone)]
pub struct AuthorizeValidatorVoteRequest {
    /// The transaction plan to authorize.
    pub validator_vote: ValidatorVoteBody,
    /// Optionally, pre-authorization data, if required by the custodian.
    pub pre_authorizations: Vec<PreAuthorization>,
}

impl DomainType for AuthorizeValidatorVoteRequest {
    type Proto = pb::AuthorizeValidatorVoteRequest;
}

impl TryFrom<pb::AuthorizeValidatorVoteRequest> for AuthorizeValidatorVoteRequest {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizeValidatorVoteRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_vote: value
                .validator_vote
                .ok_or_else(|| anyhow::anyhow!("missing validator vote"))?
                .try_into()?,
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<AuthorizeValidatorVoteRequest> for pb::AuthorizeValidatorVoteRequest {
    fn from(value: AuthorizeValidatorVoteRequest) -> pb::AuthorizeValidatorVoteRequest {
        Self {
            validator_vote: Some(value.validator_vote.into()),
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
