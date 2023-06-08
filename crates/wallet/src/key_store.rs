use anyhow::Context;
use penumbra_crypto::keys::{SeedPhrase, SpendKey};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// A wallet file storing a single spend authority.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStore {
    #[serde_as(as = "DisplayFromStr")]
    pub spend_key: SpendKey,
}

impl KeyStore {
    /// Write the wallet data to the provided path.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        if path.as_ref().exists() {
            let p = path.as_ref().to_string_lossy();
            return Err(anyhow::anyhow!(
                "Wallet file already exists, refusing to overwrite it: {}",
                &p
            ));
        }
        use std::io::Write;
        let path = path.as_ref();
        let mut file =
            std::fs::File::create(path).with_context(|| format!("can't create file {path:?}"))?;
        let data = serde_json::to_vec(self).context("can't serialize wallet")?;
        file.write_all(&data)
            .with_context(|| format!("can't write file {path:?}"))?;
        Ok(())
    }

    /// Read the wallet data from the provided path.
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        serde_json::from_slice(
            std::fs::read(path)
                .with_context(|| format!("can't read file {path:?}"))?
                .as_slice(),
        )
        .map_err(Into::into)
    }

    /// Create a new wallet.
    pub fn from_seed_phrase(seed_phrase: SeedPhrase) -> Self {
        // Currently we support a single spend authority per wallet. In the future,
        // we can derive multiple spend seeds from a single seed phrase.
        let spend_key = SpendKey::from_seed_phrase(seed_phrase, 0);

        Self { spend_key }
    }
}
