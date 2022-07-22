use std::path::Path;

use penumbra_crypto::keys::SpendKey;
use serde::{Deserialize, Serialize};

/// The path to the legacy wallet file (which actually stored a client state, not a wallet...)
pub const WALLET_FILE_NAME: &str = "penumbra_wallet.json";

/// Migrate from a legacy wallet to the current wallet format.
pub fn migrate(
    legacy_wallet_path: impl AsRef<Path>,
    custody_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let legacy_wallet_path = legacy_wallet_path.as_ref();
    let custody_path = custody_path.as_ref();

    tracing::info!("Migrating legacy wallet to new wallet format");
    let legacy_wallet: ClientState =
        serde_json::from_slice(std::fs::read(legacy_wallet_path)?.as_slice())?;

    let new_wallet = crate::KeyStore {
        spend_key: legacy_wallet.wallet.spend_key,
    };
    new_wallet.save(custody_path)?;

    // Load the new wallet, to check we really did save it:
    let new_wallet_2 = crate::KeyStore::load(custody_path)?;
    if new_wallet_2.spend_key.to_bytes().0 != new_wallet.spend_key.to_bytes().0 {
        return Err(anyhow::anyhow!("Failed to save wallet"));
    } else {
        tracing::info!("Removing legacy wallet file");
        std::fs::remove_file(legacy_wallet_path)?;
    }

    Ok(())
}

/// A legacy client state (skeleton, just enough to deserialize the keys)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientState {
    /// Key material.
    wallet: LegacyWallet,
}

/// A legacy wallet file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "serde_helpers::WalletHelper")]
#[serde(into = "serde_helpers::WalletHelper")]
pub struct LegacyWallet {
    pub spend_key: SpendKey,
}

mod serde_helpers {
    use penumbra_crypto::keys::SpendKeyBytes;
    use serde_with::serde_as;

    use super::*;

    #[serde_as]
    #[derive(Deserialize, Serialize)]
    pub struct WalletHelper {
        #[serde_as(as = "serde_with::hex::Hex")]
        spend_seed: [u8; 32],
    }

    impl From<WalletHelper> for LegacyWallet {
        fn from(w: WalletHelper) -> Self {
            Self {
                spend_key: SpendKey::from(SpendKeyBytes(w.spend_seed)),
            }
        }
    }

    impl From<LegacyWallet> for WalletHelper {
        fn from(w: LegacyWallet) -> Self {
            Self {
                spend_seed: w.spend_key.to_bytes().0,
            }
        }
    }
}
