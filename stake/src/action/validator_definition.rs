use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::validator::Validator;

/// Authenticated configuration data for a validator.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorDefinition", into = "pb::ValidatorDefinition")]
pub struct ValidatorDefinition {
    pub validator: Validator,
    pub auth_sig: Signature<SpendAuth>,
}

impl Protobuf<pb::ValidatorDefinition> for ValidatorDefinition {}

impl From<ValidatorDefinition> for pb::ValidatorDefinition {
    fn from(v: ValidatorDefinition) -> Self {
        pb::ValidatorDefinition {
            validator: Some(v.validator.into()),
            auth_sig: v.auth_sig.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::ValidatorDefinition> for ValidatorDefinition {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorDefinition) -> Result<Self, Self::Error> {
        Ok(ValidatorDefinition {
            validator: v
                .validator
                .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?
                .try_into()?,
            auth_sig: v.auth_sig.as_slice().try_into()?,
        })
    }
}
