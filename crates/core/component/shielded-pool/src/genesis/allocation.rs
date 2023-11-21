use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::shielded_pool::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A (transparent) genesis allocation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::genesis_content::Allocation",
    into = "pb::genesis_content::Allocation"
)]
pub struct Allocation {
    pub amount: Amount,
    pub denom: String,
    pub address: Address,
}

impl From<Allocation> for pb::genesis_content::Allocation {
    fn from(a: Allocation) -> Self {
        pb::genesis_content::Allocation {
            amount: Some(a.amount.into()),
            denom: a.denom,
            address: Some(a.address.into()),
        }
    }
}

impl TryFrom<pb::genesis_content::Allocation> for Allocation {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_content::Allocation) -> Result<Self, Self::Error> {
        Ok(Allocation {
            amount: msg
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount field in proto"))?
                .try_into()?,
            denom: msg.denom,
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
            .field("amount", &self.amount)
            .field("denom", &self.denom)
            .field("address", &self.address.to_string())
            .finish()
    }
}

impl DomainType for Allocation {
    type Proto = pb::genesis_content::Allocation;
}
