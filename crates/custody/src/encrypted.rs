use serde::{Deserialize, Serialize};
use serde_with::{formats::Uppercase, hex::Hex};

use crate::{soft_kms, threshold};

/// The actual inner configuration used for an encrypted configuration.
#[derive(Serialize, Deserialize)]
pub enum InnerConfig {
    SoftKms(soft_kms::Config),
    Threshold(threshold::Config),
}

impl InnerConfig {
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }

    pub fn to_bytes(self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self)?)
    }
}

/// The configuration for the encrypted custody backend.
///
/// This holds a blob of encrypted data that needs to be further deserialized into another config.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde_as(as = "Hex<Uppercase>")]
    data: Vec<u8>,
}

impl Config {
    pub fn encrypt(_password: &str, inner: InnerConfig) -> anyhow::Result<Self> {
        // TODO: encrypt with the password
        Ok(Self {
            data: inner.to_bytes()?,
        })
    }

    pub fn decrypt(self, _password: &str) -> anyhow::Result<InnerConfig> {
        Ok(InnerConfig::from_bytes(&self.data)?)
    }
}
