use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};
/// An asset denomination.
///
/// Each denomination has a unique [`asset::Id`] and base unit, and may also
/// have other display units.
#[derive(Serialize, Deserialize, Clone)]
#[serde(try_from = "pb::Denom", into = "pb::Denom")]
pub struct Denom {
    pub denom: String,
}

impl TypeUrl for Denom {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.Denom";
}

impl DomainType for Denom {
    type Proto = pb::Denom;
}

impl From<Denom> for pb::Denom {
    fn from(dn: Denom) -> Self {
        pb::Denom { denom: dn.denom }
    }
}

impl TryFrom<pb::Denom> for Denom {
    type Error = anyhow::Error;

    fn try_from(value: pb::Denom) -> Result<Self, Self::Error> {
        Ok(Denom { denom: value.denom })
    }
}
