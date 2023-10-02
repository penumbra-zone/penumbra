use penumbra_proto::penumbra::core::component::fee::v1alpha1 as pb;

use penumbra_proto::{DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::GasPrices;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FeeParameters", into = "pb::FeeParameters")]
pub struct FeeParameters {
    pub gas_prices: GasPrices,
}

impl TypeUrl for FeeParameters {
    const TYPE_URL: &'static str = "/penumbra.core.component.fee.v1alpha1.FeeParameters";
}

impl DomainType for FeeParameters {
    type Proto = pb::FeeParameters;
}

impl TryFrom<pb::FeeParameters> for FeeParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FeeParameters) -> anyhow::Result<Self> {
        Ok(FeeParameters {
            gas_prices: msg
                .gas_prices
                .ok_or_else(|| anyhow::anyhow!("proto response missing gas prices"))?
                .try_into()?,
        })
    }
}

impl From<FeeParameters> for pb::FeeParameters {
    fn from(params: FeeParameters) -> Self {
        pb::FeeParameters {
            gas_prices: Some(params.gas_prices.into()),
        }
    }
}

// TODO: defaults are implemented here as well as in the
// `pd::main`
impl Default for FeeParameters {
    fn default() -> Self {
        Self {
            gas_prices: GasPrices {
                block_space_price: 0,
                compact_block_space_price: 0,
                verification_price: 0,
                execution_price: 0,
            },
        }
    }
}
