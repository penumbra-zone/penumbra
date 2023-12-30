use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use penumbra_asset::{Balance, Value};
use penumbra_keys::Address;
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CommunityPoolOutput", into = "pb::CommunityPoolOutput")]
pub struct CommunityPoolOutput {
    pub value: Value,
    pub address: Address,
}

impl CommunityPoolOutput {
    pub fn balance(&self) -> Balance {
        // Outputs from the Community Pool require value
        -Balance::from(self.value)
    }
}

impl EffectingData for CommunityPoolOutput {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for CommunityPoolOutput {
    type Proto = pb::CommunityPoolOutput;
}

impl From<CommunityPoolOutput> for pb::CommunityPoolOutput {
    fn from(msg: CommunityPoolOutput) -> Self {
        pb::CommunityPoolOutput {
            value: Some(msg.value.into()),
            address: Some(msg.address.into()),
        }
    }
}

impl TryFrom<pb::CommunityPoolOutput> for CommunityPoolOutput {
    type Error = Error;

    fn try_from(proto: pb::CommunityPoolOutput) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;
        let address = proto
            .address
            .ok_or_else(|| anyhow::anyhow!("missing address"))?
            .try_into()
            .context("malformed address")?;

        Ok(CommunityPoolOutput { value, address })
    }
}
