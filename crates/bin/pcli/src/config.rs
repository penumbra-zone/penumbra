use std::path::Path;

use anyhow::{Context, Result};
#[cfg(feature = "ledger")]
use penumbra_sdk_custody_ledger_usb::Config as LedgerConfig;
use penumbra_sdk_stake::GovernanceKey;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

use penumbra_sdk_custody::{
    encrypted::Config as EncryptedConfig, soft_kms::Config as SoftKmsConfig,
    threshold::Config as ThresholdConfig,
};
use penumbra_sdk_keys::FullViewingKey;

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
    /// The governance custody backend to use.
    pub governance_custody: Option<GovernanceCustodyConfig>,
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

    pub fn governance_key(&self) -> GovernanceKey {
        let fvk = match &self.governance_custody {
            Some(GovernanceCustodyConfig::SoftKms(SoftKmsConfig { spend_key, .. })) => {
                spend_key.full_viewing_key()
            }
            Some(GovernanceCustodyConfig::Threshold(threshold_config)) => threshold_config.fvk(),
            Some(GovernanceCustodyConfig::Encrypted { fvk, .. }) => fvk,
            None => &self.full_viewing_key,
        };
        GovernanceKey(fvk.spend_verification_key().clone())
    }
}

/// The custody backend to use.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(tag = "backend")]
#[allow(clippy::large_enum_variant)]
pub enum CustodyConfig {
    /// A view-only client that can't sign transactions.
    ViewOnly,
    /// A software key management service.
    SoftKms(SoftKmsConfig),
    /// A manual threshold custody service.
    Threshold(ThresholdConfig),
    /// An encrypted custody service.
    Encrypted(EncryptedConfig),
    /// A custody service using an external ledger device.
    #[cfg(feature = "ledger")]
    Ledger(LedgerConfig),
}

/// The governance custody backend to use.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(tag = "backend")]
#[allow(clippy::large_enum_variant)]
pub enum GovernanceCustodyConfig {
    /// A software key management service.
    SoftKms(SoftKmsConfig),
    /// A manual threshold custody service.
    Threshold(ThresholdConfig),
    /// An encrypted custody service.
    Encrypted {
        fvk: FullViewingKey,
        config: EncryptedConfig,
    },
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
            full_viewing_key: penumbra_sdk_keys::test_keys::FULL_VIEWING_KEY.clone(),
            custody: CustodyConfig::SoftKms(SoftKmsConfig::from(
                penumbra_sdk_keys::test_keys::SPEND_KEY.clone(),
            )),
            governance_custody: None,
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
