use anyhow::{anyhow, Result};
use rand_core::CryptoRngCore;
use regex::Regex;

use crate::{
    asset::{self, Unit},
    dex::DirectedTradingPair,
    fixpoint::U128x128,
    Value,
};

use super::position::Position;

/// Helper structure for constructing a [`Position`] expressing the desire to
/// buy the `desired` value in exchange for the `offered` value.
pub struct BuyOrder {
    pub desired: Value,
    pub offered: Value,
    pub fee: u32,
}

/// Helper structure for constructing a [`Position`] expressing the desire to
/// sell the `offered` value in exchange for the `desired` value.
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

/*
impl Position {
    pub fn interpret_buy(&self) -> Option<BuyOrder> {
        // if r1 * r2 != 0, return None
        // otherwise, nonzero reserves => offered,
        // p,q imply desired,
        unimplemented!()
    }

    pub fn interpret_sell(&self) -> Option<SellOrder> {
        // if r1 * r2 != 0, return None
        // otherwise, nonzero reserves => offered,
        // p,q imply desired,
        unimplemented!()
    }
}
*/

// TODO: rendering `BuyOrder`/`SellOrder` as strings?
