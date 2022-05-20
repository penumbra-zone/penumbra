use penumbra_crypto::keys::FullViewingKeyHash;
use penumbra_proto::{custody as pb, Protobuf};
use penumbra_transaction::plan::TransactionPlan;

/// A transaction authorization request submitted to a custody service for approval.
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    /// The transaction plan to authorize.
    pub plan: TransactionPlan,
    /// Identifies the FVK (and hence the spend authorization key) to use for signing.
    pub fvk_hash: FullViewingKeyHash,
}

impl Protobuf<pb::AuthorizeRequest> for AuthorizeRequest {}

impl TryFrom<pb::AuthorizeRequest> for AuthorizeRequest {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizeRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plan: value
                .plan
                .ok_or_else(|| anyhow::anyhow!("missing plan"))?
                .try_into()?,
            fvk_hash: value
                .fvk_hash
                .ok_or_else(|| anyhow::anyhow!("missing fvk_hash"))?
                .try_into()?,
        })
    }
}

impl From<AuthorizeRequest> for pb::AuthorizeRequest {
    fn from(value: AuthorizeRequest) -> pb::AuthorizeRequest {
        Self {
            plan: Some(value.plan.into()),
            fvk_hash: Some(value.fvk_hash.into()),
        }
    }
}
