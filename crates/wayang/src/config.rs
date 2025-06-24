use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
