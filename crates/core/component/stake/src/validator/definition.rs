use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::validator::Validator;

/// Authenticated configuration data for a validator.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorDefinition", into = "pb::ValidatorDefinition")]
pub struct Definition {
    pub validator: Validator,
    pub auth_sig: Signature<SpendAuth>,
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
        let validator = v
            .validator
            .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?;
        // Validation:
        // - Website has a max length of 70 chars
        if validator.website.len() > 70 {
            anyhow::bail!("validator website field must be less than 70 characters");
        }

        // - Name has a max length of 140 chars
        if validator.name.len() > 140 {
            anyhow::bail!("validator name must be less than 140 characters");
        }

        // - Description has a max length of 280 chars
        if validator.description.len() > 280 {
            anyhow::bail!("validator description must be less than 280 characters");
        }

        Ok(Definition {
            validator: validator.try_into()?,
            auth_sig: v.auth_sig.as_slice().try_into()?,
        })
    }
}

impl EffectingData for Definition {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}
