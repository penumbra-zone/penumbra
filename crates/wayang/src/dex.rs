mod registry;
use penumbra_sdk_asset::asset::Metadata;
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use rand_core::CryptoRngCore;
pub use registry::Registry;

use anyhow::anyhow;
use penumbra_sdk_dex::{
    lp::{position::Position as PenumbraPosition, Reserves},
    DirectedTradingPair,
};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// The "symbol" providing a short name for a given asset.
///
/// For example: "USDC", "UM", etc.
///
/// You can use `as_ref` to treat a symbol as a string.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Symbol(String);

impl FromStr for Symbol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolPair {
    pub base: Symbol,
    pub quote: Symbol,
}

impl FromStr for SymbolPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base_str, quote_str) = s
            .split_once("/")
            .ok_or(anyhow!("expected string of the form 'X/Y'"))?;
        let base = Symbol::from_str(base_str)?;
        let quote = Symbol::from_str(quote_str)?;
        Ok(SymbolPair { base, quote })
    }
}

impl fmt::Display for SymbolPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base.0, self.quote.0)
    }
}

impl Serialize for SymbolPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SymbolPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SymbolPair::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct PositionShape {
    pub upper_price: f64,
    pub lower_price: f64,
    pub base_liquidity: f64,
    pub quote_liquidity: f64,
}

impl FromStr for PositionShape {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(\d+\.?\d*)/(\d+\.?\d*)\s+\[(\d+\.?\d*)\s*,\s*(\d+\.?\d*)\]")?;

        let captures = re.captures(s.trim()).ok_or_else(|| {
            anyhow::anyhow!(
                "expected format 'base_liquidity/quote_liquidity [lower_price, upper_price]'"
            )
        })?;

        let base_liquidity = captures[1]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid base liquidity: {}", &captures[1]))?;
        let quote_liquidity = captures[2]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid quote liquidity: {}", &captures[2]))?;
        let lower_price = captures[3]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid lower price: {}", &captures[3]))?;
        let upper_price = captures[4]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid upper price: {}", &captures[4]))?;

        Ok(PositionShape {
            upper_price,
            lower_price,
            base_liquidity,
            quote_liquidity,
        })
    }
}

impl Display for PositionShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} [{}, {}]",
            self.base_liquidity, self.quote_liquidity, self.lower_price, self.upper_price
        )
    }
}

const PRECISION_BITS: u64 = 60;

fn price_to_p_q(price: f64) -> anyhow::Result<(Amount, Amount)> {
    let mut p = U128x128::try_from(price)?;
    let mut q = Amount::from(1u8);
    let precision = Amount::from(1u64 << PRECISION_BITS);
    let precision_u128x128 = U128x128::from(precision);
    while !p.is_integral() {
        let Ok(new_p) = p.checked_mul(&2u8.into()) else {
            break;
        };
        let Some(new_q) = q.checked_mul(&2u8.into()) else {
            break;
        };
        if new_p > precision_u128x128 || new_q > precision {
            break;
        }
        p = new_p;
        q = new_q;
    }
    Ok((
        Amount::from_be_bytes(p.round_down().to_bytes()[..16].try_into()?),
        q,
    ))
}

fn get_exponent(meta: &Metadata) -> i32 {
    meta.default_unit().exponent().into()
}

fn convert_price(base_meta: &Metadata, quote_meta: &Metadata, price: f64) -> f64 {
    price * (10.0f64.powi(get_exponent(quote_meta) - get_exponent(base_meta)))
}

fn display_to_amount(meta: &Metadata, display: f64) -> anyhow::Result<Amount> {
    let exponent = get_exponent(meta);
    let shift = 10u128.pow(u32::try_from(exponent)?);
    Ok(Amount::from(
        (display.trunc() as u128)
            .checked_mul(shift)
            .ok_or_else(|| anyhow!("overflow when converting amount to base units"))?
            + (display.fract() * 10.0f64.powi(exponent)) as u128,
    ))
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pair: SymbolPair,
    pub shape: PositionShape,
}

impl Position {
    pub fn to_penumbra(
        self,
        rng: &mut dyn CryptoRngCore,
        registry: Registry,
    ) -> anyhow::Result<PenumbraPosition> {
        let base_meta = registry
            .lookup(&self.pair.base)
            .ok_or_else(|| anyhow!("unrecognized asset '{}'", &self.pair.base))?;
        let quote_meta = registry
            .lookup(&self.pair.quote)
            .ok_or_else(|| anyhow!("unrecognized asset '{}'", &self.pair.quote))?;
        let pair = DirectedTradingPair::new(base_meta.id(), quote_meta.id());
        let (price, fee) = {
            // m = (high + low) / 2
            // (1 + f) * m = high
            // f = high / m - 1
            let high = f64::max(self.shape.upper_price, self.shape.lower_price);
            let low = f64::min(self.shape.upper_price, self.shape.lower_price);
            let mid = (high + low) / 2.0;
            let raw_fee: f64 = high / mid - 1.0;
            let fee = (raw_fee * 10_000.0) as u32;
            (mid, fee)
        };
        let (p, q) = price_to_p_q(convert_price(&base_meta, &quote_meta, price))?;
        let r1 = display_to_amount(&base_meta, self.shape.base_liquidity)?;
        let r2 = display_to_amount(&quote_meta, self.shape.quote_liquidity)?;
        let reserves = Reserves { r1, r2 };
        Ok(PenumbraPosition::new(rng, pair, fee, p, q, reserves))
    }
}

impl FromStr for Position {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"([a-zA-Z]+/[a-zA-Z]+)\s+(.+)")?;

        let captures = re.captures(s.trim())
            .ok_or_else(|| anyhow::anyhow!("expected format 'SYMBOL1/SYMBOL2 base_liquidity/quote_liquidity [lower_price, upper_price]'"))?;

        let shape_str = &captures[2];

        let pair = SymbolPair::from_str(&captures[1])?;
        let shape = PositionShape::from_str(shape_str)?;

        Ok(Position { pair, shape })
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pair, self.shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_str() {
        let position = Position::from_str("UM/USDC 100/200 [0.8, 0.9]").unwrap();
        assert_eq!(position.pair.to_string(), "UM/USDC");
        assert_eq!(position.shape.base_liquidity, 100.0);
        assert_eq!(position.shape.quote_liquidity, 200.0);
        assert_eq!(position.shape.lower_price, 0.8);
        assert_eq!(position.shape.upper_price, 0.9);
    }

    #[test]
    fn test_position_from_str_with_decimals() {
        let position = Position::from_str("UM/USDC 100.5/200.25 [0.8, 0.9]").unwrap();
        assert_eq!(position.pair.to_string(), "UM/USDC");
        assert_eq!(position.shape.base_liquidity, 100.5);
        assert_eq!(position.shape.quote_liquidity, 200.25);
        assert_eq!(position.shape.lower_price, 0.8);
        assert_eq!(position.shape.upper_price, 0.9);
    }

    #[test]
    fn test_position_from_str_invalid_format() {
        assert!(Position::from_str("invalid").is_err());
        assert!(Position::from_str("UM/USDC").is_err());
        assert!(Position::from_str("UM 100/200 [0.8, 0.9]").is_err());
    }
}
