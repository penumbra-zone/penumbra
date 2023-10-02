use std::ops::Add;

use penumbra_num::Amount;
use penumbra_proto::{core::component::fee::v1alpha1 as pb, DomainType, TypeUrl};

/// Represents the different resources that a transaction can consume,
/// for purposes of calculating multidimensional fees based on real
/// transaction resource consumption.
pub struct Gas {
    pub block_space: u64,
    pub compact_block_space: u64,
    pub verification: u64,
    pub execution: u64,
}

impl Add for Gas {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            block_space: self.block_space + rhs.block_space,
            compact_block_space: self.compact_block_space + rhs.compact_block_space,
            verification: self.verification + rhs.verification,
            execution: self.execution + rhs.execution,
        }
    }
}

/// Expresses the price of each unit of gas in terms of the staking token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasPrices {
    pub block_space_price: u64,
    pub compact_block_space_price: u64,
    pub verification_price: u64,
    pub execution_price: u64,
}

impl GasPrices {
    pub fn price(&self, gas: &Gas) -> Amount {
        Amount::from(
            self.block_space_price * gas.block_space
                + self.compact_block_space_price * gas.compact_block_space
                + self.verification_price * gas.verification
                + self.execution_price * gas.execution,
        )
    }
}

impl TypeUrl for GasPrices {
    const TYPE_URL: &'static str = "/penumbra.core.component.fee.v1alpha1.GasPrices";
}

impl DomainType for GasPrices {
    type Proto = pb::GasPrices;
}

impl From<GasPrices> for pb::GasPrices {
    fn from(prices: GasPrices) -> Self {
        pb::GasPrices {
            block_space_price: prices.block_space_price,
            compact_block_space_price: prices.compact_block_space_price,
            verification_price: prices.verification_price,
            execution_price: prices.execution_price,
        }
    }
}

impl TryFrom<pb::GasPrices> for GasPrices {
    type Error = anyhow::Error;

    fn try_from(proto: pb::GasPrices) -> Result<Self, Self::Error> {
        Ok(GasPrices {
            block_space_price: proto.block_space_price,
            compact_block_space_price: proto.compact_block_space_price,
            verification_price: proto.verification_price,
            execution_price: proto.execution_price,
        })
    }
}
