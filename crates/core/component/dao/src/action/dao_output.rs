use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use penumbra_asset::{Balance, Value};
use penumbra_keys::Address;
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::DaoOutput", into = "pb::DaoOutput")]
pub struct DaoOutput {
    pub value: Value,
    pub address: Address,
}

impl DaoOutput {
    pub fn balance(&self) -> Balance {
        // Outputs from the DAO require value
        -Balance::from(self.value)
    }
}

impl DomainType for DaoOutput {
    type Proto = pb::DaoOutput;
}

impl From<DaoOutput> for pb::DaoOutput {
    fn from(msg: DaoOutput) -> Self {
        pb::DaoOutput {
            value: Some(msg.value.into()),
            address: Some(msg.address.into()),
        }
    }
}

impl TryFrom<pb::DaoOutput> for DaoOutput {
    type Error = Error;

    fn try_from(proto: pb::DaoOutput) -> anyhow::Result<Self, Self::Error> {
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

        Ok(DaoOutput { value, address })
    }
}
