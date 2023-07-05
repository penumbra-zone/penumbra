use penumbra_keys::keys::AccountGroupId;
use penumbra_proto::{custody::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_transaction::plan::TransactionPlan;

use crate::PreAuthorization;

/// A transaction authorization request submitted to a custody service for approval.
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    /// The transaction plan to authorize.
    pub plan: TransactionPlan,
    /// Identifies the FVK (and hence the spend authorization key) to use for signing.
    pub account_group_id: Option<AccountGroupId>,
    /// Optionally, pre-authorization data, if required by the custodian.
    pub pre_authorizations: Vec<PreAuthorization>,
}

impl TypeUrl for AuthorizeRequest {
    const TYPE_URL: &'static str = "/penumbra.custody.v1alpha1.AuthorizeRequest";
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
            account_group_id: value.account_group_id.map(TryInto::try_into).transpose()?,
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
            account_group_id: value.account_group_id.map(Into::into),
            pre_authorizations: value
                .pre_authorizations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
