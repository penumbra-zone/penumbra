use anyhow::{anyhow, Result};
use penumbra_asset::{asset, Value};
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::TradingPair;

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

    /// Checks that the specified input's asset type matches either end of this
    /// the trading function's pair. Returns `true` if so, `false` otherwise.
    pub fn matches_input(&self, input_id: asset::Id) -> bool {
        input_id == self.pair.asset_1() || input_id == self.pair.asset_2()
    }

    /// Fills a trade of an input value against this position, returning the
    /// unfilled amount of the input asset, the updated reserves, and the output
    /// amount.
    ///
    /// # Errors
    /// This method errors if:
    /// - the asset type of the input does not match either end of the
    /// `TradingPair`.
    /// - an overflow occurs during execution.
    pub fn fill(
        &self,
        input: Value,
        reserves: &Reserves,
    ) -> anyhow::Result<(Value, Reserves, Value)> {
        tracing::debug!(?input, ?reserves, "filling trade");
        if input.asset_id == self.pair.asset_1() {
            let (unfilled, new_reserves, output) = self.component.fill(input.amount, reserves)?;
            Ok((
                Value {
                    amount: unfilled,
                    asset_id: self.pair.asset_1(),
                },
                new_reserves,
                Value {
                    amount: output,
                    asset_id: self.pair.asset_2(),
                },
            ))
        } else if input.asset_id == self.pair.asset_2() {
            let flipped_reserves = reserves.flip();
            let (unfilled, new_reserves, output) = self
                .component
                .flip()
                .fill(input.amount, &flipped_reserves)?;
            Ok((
                Value {
                    amount: unfilled,
                    asset_id: self.pair.asset_2(),
                },
                new_reserves.flip(),
                Value {
                    amount: output,
                    asset_id: self.pair.asset_1(),
                },
            ))
        } else {
            Err(anyhow!(
                "input asset id {:?} did not match either end of trading pair {:?}",
                input.asset_id,
                self.pair
            ))
        }
    }

    /// Attempts to compute the input value required to produce the given output
    /// value, returning the input value and updated reserves if successful.
    /// Returns `None` if the output value exceeds the liquidity of the reserves.
    ///
    /// # Errors
    /// This method errors if:
    /// - The asset type of the output does not match either end of the reserves.
    /// - An overflow occurs during the computation.
    pub fn fill_output(
        &self,
        reserves: &Reserves,
        output: Value,
    ) -> anyhow::Result<Option<(Reserves, Value)>> {
        if output.asset_id == self.pair.asset_2() {
            Ok(self
                .component
                .fill_output(reserves, output.amount)?
                .map(|(new_reserves, input)| {
                    (
                        new_reserves,
                        Value {
                            amount: input,
                            asset_id: self.pair.asset_1(),
                        },
                    )
                }))
        } else if output.asset_id == self.pair.asset_1() {
            // Flip the reserves and the trading function...
            let flipped_reserves = reserves.flip();
            let flipped_function = self.component.flip();
            Ok(flipped_function
                .fill_output(&flipped_reserves, output.amount)?
                .map(|(new_reserves, input)| {
                    (
                        // ... then flip the reserves back.
                        new_reserves.flip(),
                        Value {
                            amount: input,
                            asset_id: self.pair.asset_2(),
                        },
                    )
                }))
        } else {
            Err(anyhow!(
                "output asset id {:?} did not match either end of trading pair {:?}",
                output.asset_id,
                self.pair
            ))
        }
    }

    pub fn orient_end(&self, end: asset::Id) -> Option<BareTradingFunction> {
        if end == self.pair.asset_2() {
            Some(self.component.clone())
        } else if end == self.pair.asset_1() {
            Some(self.component.flip())
        } else {
            None
        }
    }

    pub fn orient_start(&self, start: asset::Id) -> Option<BareTradingFunction> {
        if start == self.pair.asset_1() {
            Some(self.component.clone())
        } else if start == self.pair.asset_2() {
            Some(self.component.flip())
        } else {
            None
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
/// asset ID alone is twice the size of the trading function). Which assets correspond
/// to asset 1 and 2 is given by the canonical ordering of the pair.
///
/// The trading function `phi(R) = p*R_1 + q*R_2` is a CFMM with a constant-sum,
/// and a fee (`0 <= fee < 10_000`) expressed in basis points.
///
/// The valuations (`p`, `q`) for each asset inform the rate (or price) at which these
/// assets trade against each other.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::BareTradingFunction", into = "pb::BareTradingFunction")]
pub struct BareTradingFunction {
    /// The fee, expressed in basis points.
    ///
    /// The fee percentage of the trading function (`gamma`) is normalized
    /// according to its maximum value (10_000 bps, i.e. 100%):
    /// `gamma = (10_000 - fee) / 10_000`
    pub fee: u32,
    /// The valuation for the first asset of the pair, according to canonical ordering.
    pub p: Amount,
    /// The valuation for the second asset of the pair, according to canonical ordering.
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

    #[deprecated(note = "this method is not yet implemented")]
    pub fn fill_input(&self, _reserves: &Reserves, _delta_1: Amount) -> Option<(Reserves, Amount)> {
        unimplemented!()
    }

    /// Determine the amount of asset 1 that can be filled for a given amount of asset 2,
    /// propagating rounding error to the input amount `delta_1` rather than the output amount `lambda_2`.
    /// Returns `None` if the amount of asset 2 is greater than the reserves of asset 2.
    ///
    /// # Errors
    /// This method returns an error if an overflow occurs when computing the fillable amount of asset 1.
    #[instrument(skip(self, reserves, lambda_2))]
    pub fn fill_output(
        &self,
        reserves: &Reserves,
        lambda_2: Amount,
    ) -> anyhow::Result<Option<(Reserves, Amount)>> {
        if lambda_2 > reserves.r2 {
            tracing::debug!(?reserves, ?lambda_2, "lambda_2 > r2, no fill possible");
            return Ok(None);
        }
        // We must work backwards to infer what `delta_1` (input) correspond
        // exactly to a fill of `lambda_2 = r2`.
        // lambda_2 = effective_price * delta_1
        // and since p,q != 0, effective_price != 0:
        // delta_1 = r2 * effective_price^-1
        let fillable_delta_1 = self.convert_to_delta_1(lambda_2.into())?;

        // We burn the rouding error by apply `ceil` to delta_1:
        //
        // delta_1_star = Ceil(delta_1)
        // TODO: round_up is now fallible
        let fillable_delta_1_exact: Amount = fillable_delta_1
            .round_up()
            .expect("no overflow")
            .try_into()
            .expect("rounded up to integral value");

        let new_reserves = Reserves {
            r1: reserves.r1 + fillable_delta_1_exact,
            // We checked that lambda_2 <= reserves.r2 above.
            r2: reserves.r2 - lambda_2,
        };
        tracing::debug!(
            ?reserves,
            ?lambda_2,
            %fillable_delta_1,
            ?fillable_delta_1_exact,
            ?new_reserves,
            "computed reverse fill"
        );
        Ok(Some((new_reserves, fillable_delta_1_exact)))
    }

    /// Fills a trade of asset 1 to asset 2 against the given reserves,
    /// returning the unfilled amount of asset 1, the updated reserves, and the
    /// output amount of asset 2.
    ///
    /// # Errors
    /// This method errors if an overflow occurs when computing the trade output amount,
    /// or the fillable amount of asset 1.
    pub fn fill(&self, delta_1: Amount, reserves: &Reserves) -> Result<(Amount, Reserves, Amount)> {
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

        // The effective price is the conversion rate between `2` and `1`:
        // effective_price = (q/[gamma*p])
        // effective_price_inv = gamma*(p/q)

        // The trade output `lambda_2` is given by `effective_price * delta_1`, however, to avoid
        // rounding loss, we prefer to first compute the numerator `(gamma * delta_1 * q)`, and then
        // perform division.
        let delta_1_fp = U128x128::from(delta_1);
        let tentative_lambda_2 = self.convert_to_lambda_2(delta_1_fp)?;

        if tentative_lambda_2 <= reserves.r2.into() {
            // Observe that for the case when `tentative_lambda_2` equals
            // `reserves.r1`, rounding it down does not change anything since
            // `reserves.r1` is integral. Therefore `reserves.r1 - lambda_2 >= 0`.
            let lambda_2: Amount = tentative_lambda_2
                .round_down()
                .try_into()
                .expect("lambda_2 fits in an Amount");
            let new_reserves = Reserves {
                r1: reserves.r1 + delta_1,
                r2: reserves.r2 - lambda_2,
            };
            Ok((0u64.into(), new_reserves, lambda_2))
        } else {
            let r2: U128x128 = reserves.r2.into();
            // In this case, we don't have enough reserves to completely execute
            // the fill. So we know that `lambda_2 = r2` or that the output will
            // consist of all the reserves available.
            //
            // We must work backwards to infer what `delta_1` (input) correspond
            // exactly to a fill of `lambda_2 = r2`.
            //
            // Normally, we would have:
            //
            // lambda_2 = effective_price * delta_1
            // since lambda_2 = r2, we have:
            //
            // r2 = effective_price * delta_1, and since p,q != 0, effective_price != 0:
            // delta_1 = r2 * effective_price^-1
            let fillable_delta_1 = self.convert_to_delta_1(r2)?;

            // We burn the rouding error by apply `ceil` to delta_1:
            //
            // delta_1_star = Ceil(delta_1)
            // TODO: round_up is now fallible
            let fillable_delta_1_exact: Amount = fillable_delta_1
                .round_up()
                .expect("no overflow")
                .try_into()
                .expect("fillable_delta_1 fits in an Amount");

            // How to show that: `unfilled_amount >= 0`:
            // In this branch, we have:
            //      lambda_2 > R_2, where lambda_2 = delta_1 * effective_price:
            //      delta_1 * effective_price > R_2, in other words:
            //  <=> delta_1 > R_2 * (effective_price)^-1, in other words:
            //      delta_1 > R_2 * effective_price_inv
            //
            //  fillable_delta_1_exact = ceil(RHS) is integral (rounded), and
            //  delta_1 is integral by definition.
            //
            //  Therefore, we have:
            //
            //  delta_1 >= fillable_delta_1_exact, or in other words:
            //
            //  unfilled_amount >= 0.
            let unfilled_amount = delta_1 - fillable_delta_1_exact;

            let new_reserves = Reserves {
                r1: reserves.r1 + fillable_delta_1_exact,
                r2: 0u64.into(),
            };
            Ok((unfilled_amount, new_reserves, reserves.r2))
        }
    }

    /// Returns a byte key for this trading function with the property that the
    /// lexicographic ordering on byte keys is the same as ordering the
    /// corresponding trading functions by effective price.
    ///
    /// This allows trading functions to be indexed by price using a key-value store.
    pub fn effective_price_key_bytes(&self) -> [u8; 32] {
        self.effective_price().to_bytes()
    }

    /// Returns the inverse of the `effective_price`, in other words,
    /// the exchange rate from `asset_1` to `asset_2`:
    /// `delta_1 * effective_price_inv = lambda_2`
    pub fn effective_price_inv(&self) -> U128x128 {
        let p = U128x128::from(self.p);
        let q = U128x128::from(self.q);

        let price_ratio = (p / q).expect("q != 0 and p,q <= 2^60");
        (price_ratio * self.gamma()).expect("2^-1 <= gamma <= 1")
    }

    /// Returns the exchange rate from `asset_2` to `asset_1, inclusive
    /// of fees:
    /// `lambda_2 * effective_price = delta_1`
    pub fn effective_price(&self) -> U128x128 {
        let p = U128x128::from(self.p);
        let q = U128x128::from(self.q);

        let price_ratio = (q / p).expect("p != 0 and p,q <= 2^60");
        price_ratio.checked_div(&self.gamma()).expect("gamma != 0")
    }

    /// Converts an amount `delta_1` into `lambda_2`, using the inverse of the effective price.
    pub fn convert_to_lambda_2(&self, delta_1: U128x128) -> anyhow::Result<U128x128> {
        let lambda_2 = self.effective_price_inv() * delta_1;
        Ok(lambda_2?)
    }

    /// Converts an amount of `lambda_2` into `delta_1`, using the effective price.
    pub fn convert_to_delta_1(&self, lambda_2: U128x128) -> anyhow::Result<U128x128> {
        let delta_1 = self.effective_price() * lambda_2;
        Ok(delta_1?)
    }

    /// Returns `gamma` i.e. the fee percentage.
    /// The fee is expressed in basis points (0 <= fee < 5000), where 5000bps = 50%.
    ///
    /// ## Bounds:
    /// Since the fee `f` is bound by `0 <= < 5_000`, we have `1/2 <= gamma <= 1`.
    ///
    /// ## Examples:
    ///     * A fee of 0% (0 bps) results in a discount factor of 1.
    ///     * A fee of 30 bps (30 bps) results in a discount factor of 0.997.
    ///     * A fee of 100% (10_000bps) results in a discount factor of 0.
    pub fn gamma(&self) -> U128x128 {
        (U128x128::from(10_000 - self.fee) / U128x128::from(10_000u64)).expect("10_000 != 0")
    }

    /// Compose two trading functions together
    #[deprecated(note = "this method is not yet implemented")]
    pub fn compose(&self, _phi: BareTradingFunction) -> BareTradingFunction {
        unimplemented!()
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
    use ark_ff::Zero;
    use decaf377::Fq;
    use penumbra_asset::asset::Id;

    use super::*;

    #[test]
    /// Test that effective prices are encoded in a way that preserves their
    /// numerical ordering. Numerical ordering should transfer over lexicographic order
    /// of the encoded prices.
    fn test_trading_function_to_bytes() {
        let btf = BareTradingFunction {
            fee: 0,
            p: 2_000_000u32.into(),
            q: 1_000_000u32.into(),
        };

        assert_eq!(btf.gamma(), U128x128::from(1u64));
        assert_eq!(
            btf.effective_price_inv(),
            U128x128::ratio(btf.p, btf.q).unwrap()
        );
        let bytes1 = btf.effective_price_key_bytes();
        let price1 = btf.effective_price();

        let btf = BareTradingFunction {
            fee: 100,
            p: 2_000_000u32.into(),
            q: 1_000_000u32.into(),
        };

        // Compares the `BareTradingFunction::gamma` to a scaled ratio (10^4)
        let gamma_term =
            U128x128::ratio::<Amount>(99_000_000u64.into(), 100_000_000u64.into()).unwrap();
        assert_eq!(btf.gamma(), gamma_term,);

        let price_without_fee = U128x128::ratio(btf.p, btf.q).unwrap();
        let price_with_fee = (price_without_fee * gamma_term).unwrap();

        assert_eq!(btf.effective_price_inv(), price_with_fee);
        let bytes2 = btf.effective_price_key_bytes();
        let price2 = btf.effective_price();

        // Asserts that the lexicographic ordering of the encoded prices matches
        // their ask price ordering (smaller = better).
        //
        // price1: trading function with 0 bps fee.
        // price2: trading function with 100 bps fee.
        // price1 is "better" than price2.
        assert!(price1 < price2);
        assert!(bytes1 < bytes2);
    }

    #[test]
    /// Test that filling a position follows the asset conservation law,
    /// meaning that the R + Delta = R + Lambda
    ///
    /// There is two branches of the `BareTradingFunction::fill` method that we
    /// want to exercise. The first one is executed when there are enough reserves
    /// available to perform the fill.
    ///
    /// The second case, is when the output is constrained by the available reserves.
    fn fill_conserves_value() {
        let btf = BareTradingFunction {
            fee: 0,
            p: 1_u32.into(),
            q: 3_u32.into(),
        };

        // First, we want to test asset conservations in the case of a partial fill:
        // D_1 = 10,000,000
        // D_2 = 0
        //
        // price: p/q = 1/3, so you get 1 unit of asset 2 for 3 units of asset 1.
        //
        // L_1 = 0
        // L_2 = 3_333_333 (D_1/3)

        let old_reserves = Reserves {
            r1: 1_000_000u64.into(),
            r2: 100_000_000u64.into(),
        };

        let delta_1 = 10_000_000u64.into();
        let delta_2 = 0u64.into();
        let (lambda_1, new_reserves, lambda_2) = btf
            .fill(delta_1, &old_reserves)
            .expect("filling can't fail");
        // Conservation of value:
        assert_eq!(old_reserves.r1 + delta_1, new_reserves.r1 + lambda_1);
        assert_eq!(old_reserves.r2 + delta_2, new_reserves.r2 + lambda_2);

        // Exact amount checks:
        assert_eq!(lambda_1, 0u64.into());
        assert_eq!(lambda_2, 3_333_333u64.into());

        // Here we test trying to swap or more output than what is available in
        // the reserves:
        // lambda_1 = delta_1/3
        // lambda_2 = r2
        let old_reserves = Reserves {
            r1: 1_000_000u64.into(),
            r2: 100_000_000u64.into(),
        };
        let delta_1 = 600_000_000u64.into();
        let delta_2 = 0u64.into();

        let (lambda_1, new_reserves, lambda_2) = btf
            .fill(delta_1, &old_reserves)
            .expect("filling can't fail");
        // Conservation of value:
        assert_eq!(old_reserves.r1 + delta_1, new_reserves.r1 + lambda_1);
        assert_eq!(old_reserves.r2 + delta_2, new_reserves.r2 + lambda_2);

        // Exact amount checks:
        assert_eq!(lambda_1, 300_000_000u64.into());
        assert_eq!(lambda_2, old_reserves.r2);
        assert_eq!(new_reserves.r2, 0u64.into());
    }

    #[test]
    fn fill_bad_rounding() {
        let btf = BareTradingFunction {
            fee: 0,
            p: 12u32.into(),
            q: 10u32.into(),
        };

        let old_reserves = Reserves {
            r1: 0u64.into(),
            r2: 120u64.into(),
        };

        let delta_1 = 100u64.into();
        let (lambda_1, new_reserves, lambda_2) = btf
            .fill(delta_1, &old_reserves)
            .expect("filling can't fail");

        // Conservation of value:
        assert_eq!(old_reserves.r1 + delta_1, new_reserves.r1 + lambda_1);
        assert_eq!(old_reserves.r2 + 0u64.into(), new_reserves.r2 + lambda_2);
        // Exact amount checks:
        assert_eq!(lambda_1, 0u64.into());
        // We expect some lossy rounding here:
        assert_eq!(lambda_2, 119u64.into());
    }

    #[test]
    /// Test that the `convert_to_delta_1` and `convert_to_lambda_2` helper functions
    /// are aligned with `effective_price` and `effective_price_inv` calculations.
    fn test_conversion_helpers() {
        let btf = BareTradingFunction {
            fee: 150,
            p: 12u32.into(),
            q: 55u32.into(),
        };

        let one = U128x128::from(1u64);

        assert_eq!(btf.effective_price(), btf.convert_to_delta_1(one).unwrap());
        assert_eq!(
            btf.effective_price_inv(),
            btf.convert_to_lambda_2(one).unwrap()
        );
    }

    #[test]
    /// Test that the `TradingFunction` fills work correctly.
    fn test_fill_trading_function() {
        let a = Id(Fq::zero());
        let b = Id(Fq::ONE);
        let c = Id(Fq::ONE + Fq::ONE);

        assert!(a < b);
        assert!(b < c);

        // First, we test that everything works well when we do a fill from A to B
        // where id(A) < id(B).
        let p = Amount::from(1u64);
        let q = Amount::from(2u64);
        let phi = TradingFunction::new(TradingPair::new(a, b), 0u32, p, q);
        let reserves = Reserves {
            r1: 0u64.into(),
            r2: 100u64.into(),
        };

        let delta_1 = Value {
            amount: 200u64.into(),
            asset_id: a,
        };

        // TradingFunction::fill returns the unfilled amount, the new reserves, and the output:
        let (lambda_1, new_reserves, lambda_2) = phi.fill(delta_1, &reserves).unwrap();

        assert_eq!(lambda_1.amount, Amount::zero());
        assert_eq!(lambda_1.asset_id, delta_1.asset_id);

        assert_eq!(lambda_2.amount, reserves.r2);
        assert_eq!(lambda_2.asset_id, b);

        assert_eq!(new_reserves.r1, Amount::from(200u64));
        assert_eq!(new_reserves.r2, Amount::zero());

        // Now, we check that we fill correctly from B to A:
        // where id(A) < id(B).
        let delta_2 = Value {
            amount: 50u64.into(),
            asset_id: b,
        };

        let reserves = Reserves {
            r1: 100u64.into(),
            r2: 0u64.into(),
        };

        let (lambda_2, new_reserves, lambda_1) = phi.fill(delta_2, &reserves).unwrap();

        assert_eq!(lambda_2.amount, Amount::zero());
        assert_eq!(lambda_2.asset_id, b);

        assert_eq!(lambda_1.amount, Amount::from(100u64));
        assert_eq!(lambda_1.asset_id, a);

        assert_eq!(new_reserves.r1, Amount::zero());
        assert_eq!(new_reserves.r2, Amount::from(50u64));
    }
}
