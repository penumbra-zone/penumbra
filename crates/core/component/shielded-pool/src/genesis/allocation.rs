use penumbra_sdk_asset::{
    asset::{self, Metadata, Unit},
    Value,
};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A (transparent) genesis allocation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::genesis_content::Allocation",
    into = "pb::genesis_content::Allocation"
)]
pub struct Allocation {
    pub raw_amount: Amount,
    pub raw_denom: String,
    pub address: Address,
}

impl Allocation {
    pub fn denom(&self) -> Metadata {
        self.unit().base()
    }

    pub fn unit(&self) -> Unit {
        asset::REGISTRY.parse_unit(&self.raw_denom)
    }

    pub fn amount(&self) -> Amount {
        let unit = self.unit();
        self.raw_amount * (10u128.pow(unit.exponent().into()).into())
    }

    pub fn value(&self) -> Value {
        Value {
            amount: self.amount(),
            asset_id: self.unit().id(),
        }
    }
}

impl From<Allocation> for pb::genesis_content::Allocation {
    fn from(a: Allocation) -> Self {
        pb::genesis_content::Allocation {
            amount: Some(a.raw_amount.into()),
            denom: a.raw_denom,
            address: Some(a.address.into()),
        }
    }
}

impl TryFrom<pb::genesis_content::Allocation> for Allocation {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_content::Allocation) -> Result<Self, Self::Error> {
        Ok(Allocation {
            raw_amount: msg
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount field in proto"))?
                .try_into()?,
            raw_denom: msg.denom,
            address: msg
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address field in proto"))?
                .try_into()?,
        })
    }
}

// Implement Debug manually so we can use the Display impl for the address,
// rather than dumping all the internal address components.
impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("amount", &self.raw_amount)
            .field("denom", &self.raw_denom)
            .field("address", &self.address.to_string())
            .finish()
    }
}

impl DomainType for Allocation {
    type Proto = pb::genesis_content::Allocation;
}
