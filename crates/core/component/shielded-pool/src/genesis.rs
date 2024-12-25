use penumbra_sdk_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

mod allocation;

pub use allocation::Allocation;

use crate::params::ShieldedPoolParameters;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial token allocations.
    pub allocations: Vec<Allocation>,
    /// The initial FMD parameters.
    pub shielded_pool_params: ShieldedPoolParameters,
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            allocations: value.allocations.into_iter().map(Into::into).collect(),
            shielded_pool_params: Some(value.shielded_pool_params.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            allocations: msg
                .allocations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            shielded_pool_params: msg
                .shielded_pool_params
                .ok_or_else(|| anyhow::anyhow!("proto response missing shielded pool params"))?
                .try_into()?,
        })
    }
}

impl Default for Content {
    fn default() -> Self {
        Self {
            shielded_pool_params: ShieldedPoolParameters::default(),
            allocations: vec![
                Allocation {
                    raw_amount: 1000u128.into(),
                    raw_denom: "penumbra"
                        .parse()
                        .expect("hardcoded \"penumbra\" denom should be parseable"),
                    address: penumbra_sdk_keys::test_keys::ADDRESS_0_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    raw_amount: 100u128.into(),
                    raw_denom: "test_usd"
                        .parse()
                        .expect("hardcoded \"test_usd\" denom should be parseable"),
                    address: penumbra_sdk_keys::test_keys::ADDRESS_0_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    raw_amount: 100u128.into(),
                    raw_denom: "gm"
                        .parse()
                        .expect("hardcoded \"gm\" denom should be parseable"),
                    address: penumbra_sdk_keys::test_keys::ADDRESS_1_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
                Allocation {
                    raw_amount: 100u128.into(),
                    raw_denom: "gn"
                        .parse()
                        .expect("hardcoded \"gn\" denom should be parseable"),
                    address: penumbra_sdk_keys::test_keys::ADDRESS_1_STR
                        .parse()
                        .expect("hardcoded test address should be valid"),
                },
            ],
        }
    }
}
