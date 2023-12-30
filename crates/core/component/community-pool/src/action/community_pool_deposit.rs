use anyhow::{Context, Error};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use penumbra_asset::{Balance, Value};
// TODO: why are the CommunityPool actions not in the Community Pool protos?
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "pb::CommunityPoolDeposit",
    into = "pb::CommunityPoolDeposit"
)]
pub struct CommunityPoolDeposit {
    pub value: Value,
}

impl CommunityPoolDeposit {
    pub fn balance(&self) -> Balance {
        // Deposits into the Community Pool require value
        -Balance::from(self.value)
    }
}

impl EffectingData for CommunityPoolDeposit {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for CommunityPoolDeposit {
    type Proto = pb::CommunityPoolDeposit;
}

impl From<CommunityPoolDeposit> for pb::CommunityPoolDeposit {
    fn from(msg: CommunityPoolDeposit) -> Self {
        pb::CommunityPoolDeposit {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::CommunityPoolDeposit> for CommunityPoolDeposit {
    type Error = Error;

    fn try_from(proto: pb::CommunityPoolDeposit) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(CommunityPoolDeposit { value })
    }
}
