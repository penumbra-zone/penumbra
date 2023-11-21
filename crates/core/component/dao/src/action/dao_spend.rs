use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use penumbra_asset::{Balance, Value};
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};

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
