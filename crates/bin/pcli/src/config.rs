use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

use penumbra_custody::soft_kms::Config as SoftKmsConfig;
use penumbra_keys::FullViewingKey;

/// Configuration data for `pcli`.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PcliConfig {
    /// The URL of the gRPC endpoint used to talk to pd.
    pub grpc_url: Url,
    /// If set, use a remote view service instead of local synchronization.
    pub view_url: Option<Url>,
    /// Disable the scary "you will lose all your money" warning.
    #[serde(default, skip_serializing_if = "is_default")]
    pub disable_warning: bool,
    /// The FVK used for viewing chain data.
    #[serde_as(as = "DisplayFromStr")]
    pub full_viewing_key: FullViewingKey,
    /// The custody backend to use.
    pub custody: CustodyConfig,
}

impl PcliConfig {
    pub fn load<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path).context(format!(
            "pcli config file not found: {}. hint: run 'pcli init' to create new keys",
            &path
        ))?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(&self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// The custody backend to use.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(tag = "backend")]
pub enum CustodyConfig {
    /// A view-only client that can't sign transactions.
    ViewOnly,
    /// A software key management service.
    SoftKms(SoftKmsConfig),
}

impl Default for CustodyConfig {
    fn default() -> Self {
        Self::ViewOnly
    }
}

/// Helper function for Serde serialization, allowing us to skip serialization
/// of default config values.  Rationale: if we don't skip serialization of
/// defaults, if someone serializes a config with some default values, they're
/// "pinning" the current defaults as their choices for all time, and we have no
/// way to distinguish between fields they configured explicitly and ones they
/// passed through from the defaults. If we skip serializing default values,
/// then we know every value in the config was explicitly set.
fn is_default<T: Default + Eq>(value: &T) -> bool {
    *value == T::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_config() {
        let config = PcliConfig {
            grpc_url: Url::parse("https://grpc.testnet.penumbra.zone").unwrap(),
            disable_warning: false,
            view_url: None,
            full_viewing_key: penumbra_chain::test_keys::FULL_VIEWING_KEY.clone(),
            custody: CustodyConfig::SoftKms(SoftKmsConfig::from(
                penumbra_chain::test_keys::SPEND_KEY.clone(),
            )),
        };

        let mut config2 = config.clone();
        config2.custody = CustodyConfig::ViewOnly;
        config2.disable_warning = true;

        let toml_config = toml::to_string_pretty(&config).unwrap();
        let toml_config2 = toml::to_string_pretty(&config2).unwrap();

        println!("{}", toml_config);
        println!("{}", toml_config2);
    }
}
