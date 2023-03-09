use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::dex::{TradingPair};
use crate::fixpoint::U128x128;
use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::TradingFunction", into = "pb::TradingFunction")]
pub struct TradingFunction {
    pub component: BareTradingFunction,
    pub pair: TradingPair,
}

impl TradingFunction {
    pub fn new(pair: TradingPair, fee: u32, p: Amount, q: Amount) -> Self {
        Self {
            component: BareTradingFunction::new(fee, p, q),
            pair,
        }
    }
}

impl TryFrom<pb::TradingFunction> for TradingFunction {
    type Error = anyhow::Error;

    fn try_from(phi: pb::TradingFunction) -> Result<Self, Self::Error> {
        Ok(Self {
            component: phi
                .component
                .ok_or_else(|| anyhow::anyhow!("missing BareTradingFunction"))?
                .try_into()?,
            pair: phi
                .pair
                .ok_or_else(|| anyhow::anyhow!("missing TradingPair"))?
                .try_into()?,
        })
    }
}

impl From<TradingFunction> for pb::TradingFunction {
    fn from(phi: TradingFunction) -> Self {
        Self {
            component: Some(phi.component.into()),
            pair: Some(phi.pair.into()),
        }
    }
}

impl DomainType for TradingFunction {
    type Proto = pb::TradingFunction;
}

/// The data describing a trading function.
///
/// This implicitly treats the trading function as being between assets 1 and 2,
/// without specifying what those assets are, to avoid duplicating data (each
/// asset ID alone is twice the size of the trading function).
///
/// The trading function is `phi(R) = p*R_1 + q*R_2`.
/// This is used as a CFMM with constant `k` and fee `fee` (gamma).
///
/// NOTE: the use of floats here is a placeholder ONLY, so we can stub out the implementation,
/// and then decide what type of fixed-point, deterministic arithmetic should be used.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::BareTradingFunction", into = "pb::BareTradingFunction")]
pub struct BareTradingFunction {
    /// The fee, expressed in basis points.
    ///
    /// The equation representing the fee percentage of the trading function (`gamma`) is:
    /// `gamma = (10_000 - fee) / 10_000`.
    pub fee: u32,
    pub p: Amount,
    pub q: Amount,
}

impl BareTradingFunction {
    pub fn new(fee: u32, p: Amount, q: Amount) -> Self {
        Self { fee, p, q }
    }

    pub fn flip(&self) -> Self {
        Self {
            fee: self.fee,
            p: self.q,
            q: self.p,
        }
    }

    /// Returns a byte key for this trading function with the property that the
    /// lexicographic ordering on byte keys is the same as ordering the
    /// corresponding trading functions by effective price.
    ///
    /// This allows trading functions to be indexed by price using a key-value store.
    ///
    /// Note: Currently this uses floating point to derive the encoding, which
    /// is a placeholder and should be replaced by width-expanding polynomial arithmetic.
    pub fn effective_price_key_bytes(&self) -> [u8; 32] {
        self.effective_price().to_bytes()
    }

    /// Returns the effective price of the trading function.
    ///
    /// The effective price is the price of asset 1 in terms of asset 2 according
    /// to the trading function.
    ///
    /// This means that if there's a greater fee, the effective price is lower.
    /// Note: the float math is a placehodler
    pub fn effective_price(&self) -> U128x128 {
        (self.gamma() * U128x128::from(self.p) / U128x128::from(self.q))
            .expect("gamma < 1 and q != 0")
    }

    /// Returns the fee of the trading function, expressed as a percentage (`gamma`).
    /// Note: the float math is a placehodler
    pub fn gamma(&self) -> U128x128 {
        (U128x128::from(10_000 - self.fee) / U128x128::from(10_000u64)).expect("10_000 != 0")
    }

    /// Returns the composition of two trading functions.
    pub fn compose(&self, phi: BareTradingFunction) -> BareTradingFunction {
        let fee = self.fee * phi.fee;
        let r1 = self.p * phi.p;
        let r2 = self.q * phi.q;
        BareTradingFunction::new(fee, r1, r2)
    }
}

impl DomainType for BareTradingFunction {
    type Proto = pb::BareTradingFunction;
}

impl TryFrom<pb::BareTradingFunction> for BareTradingFunction {
    type Error = anyhow::Error;

    fn try_from(value: pb::BareTradingFunction) -> Result<Self, Self::Error> {
        Ok(Self {
            fee: value.fee,
            p: value
                .p
                .ok_or_else(|| anyhow::anyhow!("missing p"))?
                .try_into()?,
            q: value
                .q
                .ok_or_else(|| anyhow::anyhow!("missing q"))?
                .try_into()?,
        })
    }
}

impl From<BareTradingFunction> for pb::BareTradingFunction {
    fn from(value: BareTradingFunction) -> Self {
        Self {
            fee: value.fee,
            p: Some(value.p.into()),
            q: Some(value.q.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_function_to_bytes() {
        let btf = BareTradingFunction {
            fee: 0,
            p: 1_u32.into(),
            q: 2_u32.into(),
        };

        assert_eq!(btf.gamma(), U128x128::from(1u64));
        assert_eq!(btf.effective_price(), U128x128::ratio(1u64, 2u64).unwrap());
        let bytes1 = btf.effective_price_key_bytes();

        let btf = BareTradingFunction {
            fee: 100,
            p: 1_u32.into(),
            q: 1_u32.into(),
        };

        assert_eq!(btf.gamma(), U128x128::ratio(99u64, 100u64).unwrap());
        assert_eq!(
            btf.effective_price(),
            U128x128::ratio(99u64, 100u64).unwrap()
        );
        let bytes2 = btf.effective_price_key_bytes();

        assert!(bytes1 < bytes2);
    }
}
