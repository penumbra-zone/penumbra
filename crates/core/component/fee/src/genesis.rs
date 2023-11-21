use anyhow::Context;
use penumbra_proto::{penumbra::core::component::fee::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{params::FeeParameters, GasPrices};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "pb::GenesisContent", into = "pb::GenesisContent")]
pub struct Content {
    /// The initial configuration parameters for the fee component.
    pub fee_params: FeeParameters,
    /// The initial gas prices.
    pub gas_prices: GasPrices,
}

impl From<Content> for pb::GenesisContent {
    fn from(value: Content) -> Self {
        pb::GenesisContent {
            fee_params: Some(value.fee_params.into()),
            gas_prices: Some(value.gas_prices.into()),
        }
    }
}

impl TryFrom<pb::GenesisContent> for Content {
    type Error = anyhow::Error;

    fn try_from(msg: pb::GenesisContent) -> Result<Self, Self::Error> {
        Ok(Content {
            fee_params: msg
                .fee_params
                .context("fee params not present in protobuf message")?
                .try_into()?,
            gas_prices: msg
                .gas_prices
                .context("gas prices not present in protobuf message")?
                .try_into()?,
        })
    }
}

impl DomainType for Content {
    type Proto = pb::GenesisContent;
}

impl Default for Content {
    fn default() -> Self {
        Self {
            fee_params: FeeParameters::default(),
            gas_prices: GasPrices {
                block_space_price: 0,
                compact_block_space_price: 0,
                verification_price: 0,
                execution_price: 0,
            },
        }
    }
}
