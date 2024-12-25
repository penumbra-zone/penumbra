use crate::asset::Metadata;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::asset::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// An equivalent value in terms of a different numeraire.
///
/// This is used within
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "pb::EquivalentValue", into = "pb::EquivalentValue")]
pub struct EquivalentValue {
    /// The equivalent amount of the parent [`Value`] in terms of the numeraire.
    pub equivalent_amount: Amount,
    /// Metadata describing the numeraire.
    pub numeraire: Metadata,
    /// If nonzero, gives some idea of when the equivalent value was estimated (in terms of block height).
    pub as_of_height: u64,
}

impl DomainType for EquivalentValue {
    type Proto = pb::EquivalentValue;
}

impl From<EquivalentValue> for pb::EquivalentValue {
    fn from(v: EquivalentValue) -> Self {
        pb::EquivalentValue {
            equivalent_amount: Some(v.equivalent_amount.into()),
            numeraire: Some(v.numeraire.into()),
            as_of_height: v.as_of_height,
        }
    }
}

impl TryFrom<pb::EquivalentValue> for EquivalentValue {
    type Error = anyhow::Error;
    fn try_from(value: pb::EquivalentValue) -> Result<Self, Self::Error> {
        Ok(EquivalentValue {
            equivalent_amount: value
                .equivalent_amount
                .ok_or_else(|| anyhow::anyhow!("missing equivalent_amount field"))?
                .try_into()?,
            numeraire: value
                .numeraire
                .ok_or_else(|| anyhow::anyhow!("missing numeraire field"))?
                .try_into()?,
            as_of_height: value.as_of_height,
        })
    }
}
