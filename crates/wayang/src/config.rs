use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use tokio::fs;

use crate::dex::SymbolPair;

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

    pub async fn fetch(path: &Path) -> anyhow::Result<Self> {
        if !fs::try_exists(path).await? {
            tracing::info!("Config file not found, creating default.");
            fs::write(path, Self::EXAMPLE_STR.as_bytes()).await?;
        }
        tracing::info!("Loading config.");
        let data = fs::read_to_string(path).await?;
        Self::from_str(&data)
    }
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
    use crate::dex::Symbol;

    #[test]
    fn test_config_example_parsing() {
        let config = Config::default();

        assert_eq!(config.pair.base, Symbol::from_str("UM").unwrap());
        assert_eq!(config.pair.quote, Symbol::from_str("USDC").unwrap());
        assert_eq!(config.account, 1);
        assert_eq!(config.grpc_url, "https://grpc.testnet.penumbra.zone");
        assert_eq!(config.view_service, "http://localhost:8080");
    }
}
