use anyhow::{Context, Error};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

use penumbra_asset::{Balance, Value};
use penumbra_chain::{EffectHash, EffectingData};
use penumbra_keys::Address;
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::DaoOutput", into = "pb::DaoOutput")]
pub struct DaoOutput {
    pub value: Value,
    pub address: Address,
}

impl EffectingData for DaoOutput {
    fn effect_hash(&self) -> EffectHash {
        let effecting_data: pb::DaoOutput = self.clone().into();

        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:daooutput")
            .to_state();
        state.update(&effecting_data.encode_to_vec());

        EffectHash(*state.finalize().as_array())
    }
}

impl DaoOutput {
    pub fn balance(&self) -> Balance {
        // Outputs from the DAO require value
        -Balance::from(self.value)
    }
}

impl TypeUrl for DaoOutput {
    const TYPE_URL: &'static str = "/penumbra.core.governance.v1alpha1.DaoOutput";
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
