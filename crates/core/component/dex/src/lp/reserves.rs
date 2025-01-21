use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};

use crate::TradingPair;

use super::position::MAX_RESERVE_AMOUNT;

/// The reserves of a position.
///
/// Like a position, this implicitly treats the trading function as being
/// between assets 1 and 2, without specifying what those assets are, to avoid
/// duplicating data (each asset ID alone is four times the size of the
/// reserves).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reserves {
    pub r1: Amount,
    pub r2: Amount,
}

impl Reserves {
    pub fn check_bounds(&self) -> anyhow::Result<()> {
        if self.r1.value() > MAX_RESERVE_AMOUNT || self.r2.value() > MAX_RESERVE_AMOUNT {
            anyhow::bail!(format!(
                "Reserve amounts are out-of-bounds (limit: {MAX_RESERVE_AMOUNT})"
            ))
        } else {
            // Both reserves cannot be empty.
            if self.r1.value() == 0 && self.r2.value() == 0 {
                anyhow::bail!("initial reserves must provision some amount of either asset",);
            }

            Ok(())
        }
    }

    /// Augment `self` with type information to get a typed `Balance`.
    pub fn balance(&self, pair: &TradingPair) -> Balance {
        let r1 = Value {
            amount: self.r1,
            asset_id: pair.asset_1(),
        };

        let r2 = Value {
            amount: self.r2,
            asset_id: pair.asset_2(),
        };

        Balance::from(r1) + r2
    }

    /// Flip the reserves
    pub fn flip(&self) -> Reserves {
        Self {
            r1: self.r2,
            r2: self.r1,
        }
    }

    /// Return zero reserves.
    pub fn zero() -> Self {
        Self {
            r1: Amount::zero(),
            r2: Amount::zero(),
        }
    }
}

impl Default for Reserves {
    fn default() -> Self {
        Self::zero()
    }
}

impl DomainType for Reserves {
    type Proto = pb::Reserves;
}

impl TryFrom<pb::Reserves> for Reserves {
    type Error = anyhow::Error;

    fn try_from(value: pb::Reserves) -> Result<Self, Self::Error> {
        Ok(Self {
            r1: value
                .r1
                .ok_or_else(|| anyhow::anyhow!("missing r1"))?
                .try_into()?,
            r2: value
                .r2
                .ok_or_else(|| anyhow::anyhow!("missing r2"))?
                .try_into()?,
        })
    }
}

impl From<Reserves> for pb::Reserves {
    fn from(value: Reserves) -> Self {
        Self {
            r1: Some(value.r1.into()),
            r2: Some(value.r2.into()),
        }
    }
}
