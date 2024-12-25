use penumbra_sdk_proto::penumbra::core::component::fee::v1 as pb;

use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

use crate::GasPrices;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(try_from = "pb::FeeParameters", into = "pb::FeeParameters")]
pub struct FeeParameters {
    pub fixed_gas_prices: GasPrices,
    pub fixed_alt_gas_prices: Vec<GasPrices>,
}

impl DomainType for FeeParameters {
    type Proto = pb::FeeParameters;
}

impl TryFrom<pb::FeeParameters> for FeeParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FeeParameters) -> anyhow::Result<Self> {
        Ok(FeeParameters {
            fixed_gas_prices: msg.fixed_gas_prices.unwrap_or_default().try_into()?,
            fixed_alt_gas_prices: msg
                .fixed_alt_gas_prices
                .into_iter()
                .map(|p| p.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<FeeParameters> for pb::FeeParameters {
    fn from(params: FeeParameters) -> Self {
        pb::FeeParameters {
            fixed_gas_prices: Some(params.fixed_gas_prices.into()),
            fixed_alt_gas_prices: params
                .fixed_alt_gas_prices
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
