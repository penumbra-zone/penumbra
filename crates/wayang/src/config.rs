use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// The "symbol" providing a short name for a given asset.
///
/// For example: "USDC", "UM", etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Symbol(String);

impl FromStr for Symbol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The pair on which to provide liquidity.
    pub pair: SymbolPair,
    /// The account to use for funds.
    ///
    /// This should be a separate account from the main account (so, > 0), so
    /// that it requires intention to place funds in here. Ideally, the only activity
    /// on this account should be a particular active strategy, which avoids issues
    /// with concurrency, and makes tracking profit easier.
    pub account: u32,
    /// The ratio of funds that can be used to provide liquidity on this pair. In [0, 1].
    ///
    /// For example, if providing UM / USDC liquidity, and this value is set to 20%, then
    /// up to 20% of the UM or 20% of the USDC may be used.
    /// This is also a ceiling, and not a floor, so less may be in use at any point
    /// in time.
    pub liquidity_ratio: f64,
    /// The URL we'll use to contact an RPC provider (e.g. a full node).
    pub grpc_url: String,
    /// The URL we'll use to contact a view service (e.g. `pclientd`).
    pub view_service: String,
}

impl Config {
    /// A full example config file.
    ///
    /// This is a string, so that it can include comments.
    pub const EXAMPLE_STR: &'static str = include_str!("../config_example.toml");
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        toml::from_str(s).map_err(|e| anyhow!("failed to parse config: {}", e))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_str(Self::EXAMPLE_STR).expect("Failed to parse example Config.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_example_parsing() {
        let config = Config::default();

        assert_eq!(config.pair.base.0, "UM");
        assert_eq!(config.pair.quote.0, "USDC");
        assert_eq!(config.account, 1);
        assert_eq!(config.liquidity_ratio, 0.8);
        assert_eq!(config.grpc_url, "https://grpc.testnet.penumbra.zone");
        assert_eq!(config.view_service, "http://localhost:8080");
    }
}
