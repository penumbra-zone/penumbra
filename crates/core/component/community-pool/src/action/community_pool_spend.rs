use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use penumbra_asset::{Balance, Value};
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CommunityPoolSpend", into = "pb::CommunityPoolSpend")]
pub struct CommunityPoolSpend {
    pub value: Value,
}

impl EffectingData for CommunityPoolSpend {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl CommunityPoolSpend {
    pub fn balance(&self) -> Balance {
        // Spends from the Community Pool produce value
        Balance::from(self.value)
    }
}

impl DomainType for CommunityPoolSpend {
    type Proto = pb::CommunityPoolSpend;
}

impl From<CommunityPoolSpend> for pb::CommunityPoolSpend {
    fn from(msg: CommunityPoolSpend) -> Self {
        pb::CommunityPoolSpend {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::CommunityPoolSpend> for CommunityPoolSpend {
    type Error = Error;

    fn try_from(proto: pb::CommunityPoolSpend) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(CommunityPoolSpend { value })
    }
}
