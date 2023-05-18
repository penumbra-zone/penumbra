use anyhow::{anyhow, Result};
use rand_core::CryptoRngCore;
use regex::Regex;

use crate::{
    asset::{self, Unit},
    dex::DirectedTradingPair,
    fixpoint::U128x128,
    Amount, Value,
};

use super::position::Position;

/// Helper structure for constructing a [`Position`] expressing the desire to
/// buy the `desired` value in exchange for the `offered` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyOrder {
    pub desired: Value,
    pub offered: Value,
    pub fee: u32,
}

/// Helper structure for constructing a [`Position`] expressing the desire to
/// sell the `offered` value in exchange for the `desired` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellOrder {
    pub offered: Value,
    pub desired: Value,
    pub fee: u32,
}

/// This doesn't parse the values yet, because we need to inspect their units.
fn parse_parts(input: &str) -> Result<(&str, &str, u32)> {
    let (trade_part, fee_part) = match input.split_once('/') {
        Some((trade_part, fee_part)) => (trade_part, fee_part),
        None => (input, "0bps"),
    };

    let Some((val1, val2)) = trade_part.split_once('@') else {
        return Err(anyhow!("could not parse trade string {}", input));
    };

    let fee = match fee_part.strip_suffix("bps") {
        Some(fee) => fee.parse::<u32>()?,
        None => return Err(anyhow!("could not parse fee string {}", fee_part)),
    };

    Ok((val1, val2, fee))
}

fn extract_unit(input: &str) -> Result<Unit> {
    let unit_re = Regex::new(r"[0-9.]+([^0-9.].*+)$")?;
    if let Some(captures) = unit_re.captures(input) {
        let unit = captures.get(1).expect("matched regex").as_str();
        Ok(asset::REGISTRY.parse_unit(unit))
    } else {
        Err(anyhow!("could not extract unit from {}", input))
    }
}

impl BuyOrder {
    /// Eventually we'll need to plumb in an asset::Cache so this isn't FromStr
    pub fn parse_str(input: &str) -> Result<Self> {
        let (desired_str, price_str, fee) = parse_parts(input)?;

        let desired_unit = extract_unit(desired_str)?;
        let desired = desired_str.parse::<Value>()?;
        let price = price_str.parse::<Value>()?;

        // In, e.g., 100mpenumbra@1.2gm, we're expressing the desire to:
        // - buy 100_000 upenumbra (absolute value)
        // - at a price of 1.2 gm per mpenumbra
        // So, our offered amount is 1.2 gm * 100 = 120 gm, or
        // 1_200_000 ugm * (100_000 upenumbra / 1_000 (mpenumbra per upenumbra)).

        let price_amount = U128x128::from(price.amount); // e.g., 1_200_000
        let desired_amount = U128x128::from(desired.amount); // e.g., 100_000
        let desired_unit_amount = U128x128::from(desired_unit.unit_amount()); // e.g., 1_000

        let offered_amount = ((price_amount * desired_amount) / desired_unit_amount)?
            .round_up()
            .try_into()
            .expect("rounded to integer");

        let offered = Value {
            amount: offered_amount,
            asset_id: price.asset_id,
        };

        Ok(BuyOrder {
            desired,
            offered,
            fee,
        })
    }

    /// Returns a formatted representation of the price component.  This is
    /// returned as a string because it implicitly depends on the unit of the
    /// offered asset, so shouldn't be used for computation; it's split out for
    /// use in tables.
    ///
    /// Errors if the assets in `self` aren't in `cache`.
    pub fn price_str(&self, cache: &asset::Cache) -> Result<String> {
        let desired_unit = cache
            .get(&self.desired.asset_id)
            .map(|d| d.default_unit())
            .ok_or_else(|| anyhow!("unknown asset {}", self.desired.asset_id))?;

        // When parsing, we have
        // offered_amount = ceil(price_amount * desired_amount / desired_unit_amount)
        // We want to compute price_amount
        //   ignoring rounding, this is
        //   price_amount = offered_amount * desired_unit_amount / desired_amount
        let offered_amount = U128x128::from(self.offered.amount);
        let desired_amount = U128x128::from(self.desired.amount);
        let desired_unit_amount = U128x128::from(desired_unit.unit_amount());

        let price_amount: Amount = ((offered_amount * desired_unit_amount) / desired_amount)?
            // TODO: Is this the correct rounding behavior? Should we expect this to round-trip exactly?
            .round_up()
            .try_into()
            .expect("rounded to integer");

        let price_str = Value {
            amount: price_amount,
            asset_id: self.offered.asset_id,
        }
        .format(&cache);

        Ok(price_str)
    }

    /// Formats this `BuyOrder` as a string.
    pub fn format(&self, cache: &asset::Cache) -> Result<String> {
        let price_str = self.price_str(cache)?;
        let desired_str = self.desired.format(&cache);

        if self.fee != 0 {
            Ok(format!("{}@{}/{}bps", desired_str, price_str, self.fee))
        } else {
            Ok(format!("{}@{}", desired_str, price_str))
        }
    }
}

impl SellOrder {
    /// Eventually we'll need to plumb in an asset::Cache so this isn't FromStr
    pub fn parse_str(input: &str) -> Result<Self> {
        let (offered_str, price_str, fee) = parse_parts(input)?;

        let offered_unit = extract_unit(offered_str)?;
        let offered = offered_str.parse::<Value>()?;
        let price = price_str.parse::<Value>()?;

        // In, e.g., 100mpenumbra@1.2gm, we're expressing the desire to:
        // - sell 100_000 upenumbra (absolute value)
        // - at a price of 1.2 gm per mpenumbra
        // So, our desired amount is 1.2 gm * 100 = 120 gm, or
        // 1_200_000 ugm * (100_000 upenumbra / 1_000 (mpenumbra per upenumbra)).

        let price_amount = U128x128::from(price.amount); // e.g., 1_200_000
        let offered_amount = U128x128::from(offered.amount); // e.g., 100_000
        let offered_unit_amount = U128x128::from(offered_unit.unit_amount()); // e.g., 1_000

        let desired_amount = ((price_amount * offered_amount) / offered_unit_amount)?
            .round_up()
            .try_into()
            .expect("rounded to integer");

        let desired = Value {
            amount: desired_amount,
            asset_id: price.asset_id,
        };

        Ok(SellOrder {
            offered,
            desired,
            fee,
        })
    }

    /// Returns a formatted representation of the price component.  This is
    /// returned as a string because it implicitly depends on the unit of the
    /// offered asset, so shouldn't be used for computation; it's split out for
    /// use in tables.
    ///
    /// Errors if the assets in `self` aren't in `cache`.
    pub fn price_str(&self, cache: &asset::Cache) -> Result<String> {
        let offered_unit = cache
            .get(&self.offered.asset_id)
            .map(|d| d.default_unit())
            .ok_or_else(|| anyhow!("unknown asset {}", self.offered.asset_id))?;

        // When parsing, we have
        // desired_amount = ceil(price_amount * offered_amount / offered_unit_amount)
        // We want to compute price_amount
        //   ignoring rounding, this is
        //   price_amount = desired_amount * offered_unit_amount / offered_amount
        let offered_amount = U128x128::from(self.offered.amount);
        let desired_amount = U128x128::from(self.desired.amount);
        let offered_unit_amount = U128x128::from(offered_unit.unit_amount());

        let price_amount: Amount = ((desired_amount * offered_unit_amount) / offered_amount)?
            // TODO: Is this the correct rounding behavior? Should we expect this to round-trip exactly?
            .round_up()
            .try_into()
            .expect("rounded to integer");

        let price_str = Value {
            amount: price_amount,
            asset_id: self.desired.asset_id,
        }
        .format(&cache);

        Ok(price_str)
    }

    /// Formats this `SellOrder` as a string.
    pub fn format(&self, cache: &asset::Cache) -> Result<String> {
        let price_str = self.price_str(cache)?;
        let offered_str = self.offered.format(&cache);

        if self.fee != 0 {
            Ok(format!("{}@{}/{}bps", offered_str, price_str, self.fee))
        } else {
            Ok(format!("{}@{}", offered_str, price_str))
        }
    }
}

fn into_position_inner<R: CryptoRngCore>(
    offered: Value,
    desired: Value,
    fee: u32,
    rng: R,
) -> Position {
    // We want to compute `p` and `q` that interpolate between two reserves states:
    // (r1, r2) = (offered, 0) ; k = r1 * p + r2 * q = offered * p
    // (r1, r2) = (0, desired) ; k = r1 * p + r2 * q = desired * q
    // Setting p = desired, q = offered gives k = offered * desired in both cases.
    let p = desired.amount;
    let q = offered.amount;
    Position::new(
        rng,
        DirectedTradingPair {
            start: offered.asset_id,
            end: desired.asset_id,
        },
        fee,
        p,
        q,
        super::Reserves {
            r1: offered.amount,
            r2: 0u64.into(),
        },
    )
}

impl BuyOrder {
    pub fn into_position<R: CryptoRngCore>(&self, rng: R) -> Position {
        into_position_inner(self.offered, self.desired, self.fee, rng)
    }
}

impl SellOrder {
    pub fn into_position<R: CryptoRngCore>(&self, rng: R) -> Position {
        into_position_inner(self.offered, self.desired, self.fee, rng)
    }
}

// TODO: maybe useful in cleaning up cli rendering?
// unsure if we can get an exact round trip

impl Position {
    fn interpret_inner(&self) -> Option<(Value, Value)> {
        // if r1 * r2 != 0, return None
        // otherwise, nonzero reserves => offered,
        // p,q imply desired,
        let offered = if self.reserves.r1 == 0u64.into() {
            Value {
                amount: self.reserves.r2,
                asset_id: self.phi.pair.asset_2(),
            }
        } else if self.reserves.r2 == 0u64.into() {
            Value {
                amount: self.reserves.r1,
                asset_id: self.phi.pair.asset_1(),
            }
        } else {
            return None;
        };
        // The "desired" amount is the fill against the reserves.
        // However, we don't want to account for fees here, so make a feeless copy of
        // the trading function.
        let mut feeless_phi = self.phi.clone();
        feeless_phi.component.fee = 0;
        let (_new_reserves, desired) = feeless_phi
            .fill_output(&self.reserves, offered)
            .expect("asset types match")
            .expect("supplied exact reserves");

        Some((offered, desired))
    }

    /// Attempts to interpret this position as a "buy order".
    ///
    /// If both of the reserves are nonzero, returns None.
    pub fn interpret_as_buy(&self) -> Option<BuyOrder> {
        self.interpret_inner().map(|(offered, desired)| BuyOrder {
            offered,
            desired,
            fee: self.phi.component.fee,
        })
    }

    /// Attempts to interpret this position as a "sell order".
    ///
    /// If both of the reserves are nonzero, returns None.
    pub fn interpret_as_sell(&self) -> Option<SellOrder> {
        self.interpret_inner().map(|(offered, desired)| SellOrder {
            offered,
            desired,
            fee: self.phi.component.fee,
        })
    }

    /// Interprets a position with mixed reserves as a pair of "sell orders".
    pub fn interpret_as_mixed(&self) -> Option<(SellOrder, SellOrder)> {
        let mut split1 = self.clone();
        let mut split2 = self.clone();

        if split1.reserves.r2 == 0u64.into() {
            return None;
        }
        split1.reserves.r2 = 0u64.into();

        if split2.reserves.r1 == 0u64.into() {
            return None;
        }
        split2.reserves.r1 = 0u64.into();

        Some((
            split1.interpret_as_sell().expect("r2 is zero"),
            split2.interpret_as_sell().expect("r1 is zero"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_buy_order_basic() {
        // TODO: should have a way to build an asset::Cache for known assets
        let mut cache = asset::Cache::default();
        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        cache.extend([gm.base(), gn.base()]);

        let buy_str_1 = "123.444gm@2gn/10bps";
        let buy_order_1 = BuyOrder::parse_str(buy_str_1).unwrap();
        assert_eq!(
            buy_order_1,
            BuyOrder {
                desired: Value {
                    amount: 123_444000u64.into(),
                    asset_id: gm.id()
                },
                offered: Value {
                    amount: 246_888000u64.into(),
                    asset_id: gn.id()
                },
                fee: 10,
            }
        );

        let buy_formatted_1 = buy_order_1.format(&cache).unwrap();
        assert_eq!(buy_formatted_1, buy_str_1);

        let buy_position_1 = buy_order_1.into_position(rand::thread_rng());
        let buy_position_as_order_1 = buy_position_1.interpret_as_buy().unwrap();
        let buy_position_formatted_1 = buy_position_as_order_1.format(&cache).unwrap();

        assert_eq!(buy_position_as_order_1, buy_order_1);
        assert_eq!(buy_position_formatted_1, buy_str_1);
    }

    #[test]
    fn parse_sell_order_basic() {
        // TODO: should have a way to build an asset::Cache for known assets
        let mut cache = asset::Cache::default();
        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        cache.extend([gm.base(), gn.base()]);

        let sell_str_1 = "123.444gm@2gn/10bps";
        let sell_order_1 = SellOrder::parse_str(sell_str_1).unwrap();
        assert_eq!(
            sell_order_1,
            SellOrder {
                desired: Value {
                    amount: 246_888000u64.into(),
                    asset_id: gn.id()
                },
                offered: Value {
                    amount: 123_444000u64.into(),
                    asset_id: gm.id()
                },
                fee: 10,
            }
        );

        let sell_formatted_1 = sell_order_1.format(&cache).unwrap();
        assert_eq!(sell_formatted_1, sell_str_1);

        let sell_position_1 = sell_order_1.into_position(rand::thread_rng());
        let sell_position_as_order_1 = sell_position_1.interpret_as_sell().unwrap();
        let sell_position_formatted_1 = sell_position_as_order_1.format(&cache).unwrap();

        assert_eq!(sell_position_as_order_1, sell_order_1);
        assert_eq!(sell_position_formatted_1, sell_str_1);
    }
}
