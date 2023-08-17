use anyhow::{Context, Error};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use penumbra_asset::{Balance, Value};
use penumbra_chain::{EffectHash, EffectingData};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::DaoSpend", into = "pb::DaoSpend")]
pub struct DaoSpend {
    pub value: Value,
}

impl DaoSpend {
    pub fn balance(&self) -> Balance {
        // Spends from the DAO produce value
        Balance::from(self.value)
    }
}

impl TypeUrl for DaoSpend {
    const TYPE_URL: &'static str = "/penumbra.core.governance.v1alpha1.DaoSpend";
}

impl DomainType for DaoSpend {
    type Proto = pb::DaoSpend;
}

impl From<DaoSpend> for pb::DaoSpend {
    fn from(msg: DaoSpend) -> Self {
        pb::DaoSpend {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::DaoSpend> for DaoSpend {
    type Error = Error;

    fn try_from(proto: pb::DaoSpend) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(DaoSpend { value })
    }
}
