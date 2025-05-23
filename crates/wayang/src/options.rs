use anyhow::anyhow;
use std::str::FromStr;

/// The "symbol" providing a short name for a given asset.
///
/// For example: "USDC", "UM", etc.
#[derive(Clone, Debug)]
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
    base: Symbol,
    quote: Symbol,
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

#[derive(clap::Parser)]
pub struct Options {
    /// The pair on which to provide liquidity.
    ///
    /// This should be provided as a string such as 'UM/USDC', with the first part
    /// the symbol of the base asset, and the second being the symbol of the quote asset.
    #[clap(long)]
    pub pair: SymbolPair,
}
