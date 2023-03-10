use anyhow::anyhow;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::dex::TradingPair;
use crate::fixpoint::U128x128;
use crate::Amount;
use crate::Value;

use super::Reserves;

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

    /// Fills a trade of an input value against this position, returning the
    /// unfilled amount of the input asset, the updated reserves, and the output
    /// amount.
    ///
    /// Errors if the asset type of the input does not match either end of this
    /// `TradingFunction`'s `TradingPair`.
    pub fn fill(
        &self,
        input: Value,
        reserves: &Reserves,
    ) -> anyhow::Result<(Value, Reserves, Value)> {
        if input.asset_id == self.pair.asset_1() {
            let (unfilled, new_reserves, output) = self.component.fill(input.amount, reserves);
            return Ok((
                Value {
                    amount: unfilled,
                    asset_id: self.pair.asset_1(),
                },
                new_reserves,
                Value {
                    amount: output,
                    asset_id: self.pair.asset_2(),
                },
            ));
        } else if input.asset_id == self.pair.asset_2() {
            let flipped_reserves = reserves.flip();
            let (unfilled, new_reserves, output) =
                self.component.flip().fill(input.amount, &flipped_reserves);
            return Ok((
                Value {
                    amount: unfilled,
                    asset_id: self.pair.asset_2(),
                },
                new_reserves.flip(),
                Value {
                    amount: output,
                    asset_id: self.pair.asset_1(),
                },
            ));
        } else {
            Err(anyhow!(
                "input asset id {:?} did not match either end of trading pair {:?}",
                input.asset_id,
                self.pair
            ))
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

    /// Fills a trade of asset 1 to asset 2 against the given reserves,
    /// returning the unfilled amount of asset 1, the updated reserves, and the
    /// output amount of asset 2.
    pub fn fill(&self, delta_1: Amount, reserves: &Reserves) -> (Amount, Reserves, Amount) {
        // We distinguish two cases, which only differ in their rounding
        // behavior.
        //
        // If the desired fill is less than the original reserves, we want to
        // work "forward" from the input amount `delta_1` to the output amount
        // `lambda_2`, consuming exactly `delta_1` and rounding `lambda_2`
        // (down, so that we're burning the rounding error).
        //
        // If the desired fill is greater than the original reserves, however,
        // we want to work "backward" from the available reserves `R_2` (the
        // "max fill") to the input amount `delta_1`, producing exactly
        // `lambda_2 = R_2` output and rounding `delta_1` (up, so that we're
        // burning the rounding error).
        //
        // We want to be sure that in either case, we only round once, and derive
        // other quantities exactly from the rounded quantity. This ensures
        // conservation of value.
        //
        // This also ensures that we cleanly fill the position, rather than
        // leaving some dust amount of reserves in it. Otherwise, we might try
        // executing against it again on a subsequent iteration, even though it
        // was essentially filled.
        let tentative_lambda_2 = (self.effective_price() * U128x128::from(delta_1)).unwrap();
        if tentative_lambda_2 <= reserves.r2.into() {
            // Observe that for the case when `tentative_lambda_2` equals
            // `reserves.r1`, rounding it down does not change anything since
            // `reserves.r1` is integral. Therefore `reserves.r1 - lambda_2 >= 0`.
            let lambda_2: Amount = tentative_lambda_2.round_down().try_into().unwrap();
            let new_reserves = Reserves {
                r1: reserves.r1 + delta_1,
                r2: reserves.r2 - lambda_2,
            };
            (0u64.into(), new_reserves, lambda_2)
        } else {
            let r2: U128x128 = reserves.r2.into();
            let fillable_delta_1 = (r2 / self.effective_price()).unwrap();

            let fillable_delta_1_exact: Amount = fillable_delta_1.round_up().try_into().unwrap();
            // We know that unfilled_amount >= 0. Why?
            // In this branch, we have delta_1 * (q/p) * gamma > R_2
            //                     <=> R_2 * (p/q) * (1/gamma) < delta_1
            // Since fillable_delta_1_exact = ceil(LHS) and the RHS is already integral,
            // we know that ceil(LHS) <= RHS and so fillable_delta_1_exact <= delta_1.
            let unfilled_amount = delta_1 - fillable_delta_1_exact;

            let new_reserves = Reserves {
                r1: reserves.r1 + fillable_delta_1_exact,
                r2: 0u64.into(),
            };
            (unfilled_amount, new_reserves, reserves.r2)
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
    /// The effective price is the price of asset 2 in terms of asset 1 according
    /// to the trading function.
    ///
    /// This means that if there's a greater fee, the effective price is lower.
    pub fn effective_price(&self) -> U128x128 {
        (self.gamma() * U128x128::from(self.q) / U128x128::from(self.p))
            .expect("gamma < 1 and p != 0")
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
            p: 2_u32.into(),
            q: 1_u32.into(),
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

    #[test]
    fn fill_conserves_value() {
        let btf = BareTradingFunction {
            fee: 0,
            p: 3_u32.into(),
            q: 1_u32.into(),
        };

        let old_reserves = Reserves {
            r1: 1_000_000u64.into(),
            r2: 100_000_000u64.into(),
        };

        let input_a = 10_000_000u64.into();
        let (unfilled_a, new_reserves_a, output_a) = btf.fill(input_a, &old_reserves);
        // Conservation of value:
        assert_eq!(old_reserves.r1 + input_a, new_reserves_a.r1 + unfilled_a);
        assert_eq!(old_reserves.r2 + 0u64.into(), new_reserves_a.r2 + output_a);
        // Exact amount checks:
        assert_eq!(output_a, 3_333_333u64.into()); // 10.0 -> 3.33...
        assert_eq!(unfilled_a, 0u64.into());

        let input_b = 600_000_000u64.into();
        // Conservation of value:
        let (unfilled_b, new_reserves_b, output_b) = btf.fill(input_b, &old_reserves);
        assert_eq!(old_reserves.r1 + input_b, new_reserves_b.r1 + unfilled_b);
        assert_eq!(old_reserves.r2 + 0u64.into(), new_reserves_b.r2 + output_b);
        // Exact amount checks:
        assert_eq!(output_b, old_reserves.r2); // Exact fill of position
        assert_eq!(unfilled_b, 299_999_999u64.into()); // rounding error is burned
    }
}
