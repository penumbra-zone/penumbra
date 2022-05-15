use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_proto::{transaction as pb, Protobuf};

use crate::AuthHash;

/// Authorization data returned in response to a
/// [`TransactionDescription`](crate::TransactionDescription).
#[derive(Clone, Debug)]
pub struct AuthorizationData {
    /// The computed authorization hash for the approved transaction.
    pub auth_hash: AuthHash,
    /// The required spend authorization signatures, returned
    /// in the same order as the Spend actions in the original request.
    pub spend_auths: Vec<Signature<SpendAuth>>,
}

impl Protobuf<pb::AuthorizationData> for AuthorizationData {}

impl From<AuthorizationData> for pb::AuthorizationData {
    fn from(msg: AuthorizationData) -> Self {
        Self {
            auth_hash: Some(msg.auth_hash.into()),
            spend_auths: msg.spend_auths.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::AuthorizationData> for AuthorizationData {
    type Error = anyhow::Error;
    fn try_from(value: pb::AuthorizationData) -> Result<Self, Self::Error> {
        Ok(Self {
            auth_hash: value
                .auth_hash
                .ok_or_else(|| anyhow::anyhow!("missing auth_hash"))?
                .try_into()?,
            spend_auths: value
                .spend_auths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
