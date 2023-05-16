use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::validator::Validator;

/// Authenticated configuration data for a validator.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorDefinition", into = "pb::ValidatorDefinition")]
pub struct Definition {
    pub validator: Validator,
    pub auth_sig: Signature<SpendAuth>,
}

impl TypeUrl for Definition {
    const TYPE_URL: &'static str = "/penumbra.core.stake.v1alpha1.ValidatorDefinition";
}

impl DomainType for Definition {
    type Proto = pb::ValidatorDefinition;
}

impl From<Definition> for pb::ValidatorDefinition {
    fn from(v: Definition) -> Self {
        pb::ValidatorDefinition {
            validator: Some(v.validator.into()),
            auth_sig: v.auth_sig.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::ValidatorDefinition> for Definition {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorDefinition) -> Result<Self, Self::Error> {
        Ok(Definition {
            validator: v
                .validator
                .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?
                .try_into()?,
            auth_sig: v.auth_sig.as_slice().try_into()?,
        })
    }
}
