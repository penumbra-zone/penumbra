use serde::{Deserialize, Serialize};

use penumbra_proto::core::keys::v1alpha1;
use penumbra_proto::{penumbra::core::keys::v1alpha1 as pb, serializers::bech32str};

/// The hash of a full viewing key, used as an account identifier.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "pb::WalletId", into = "pb::WalletId")]
pub struct WalletId(pub [u8; 32]);

impl TryFrom<v1alpha1::WalletId> for WalletId {
    type Error = anyhow::Error;

    fn try_from(value: v1alpha1::WalletId) -> Result<Self, Self::Error> {
        Ok(WalletId(
            value
                .inner
                .try_into()
                .map_err(|_| anyhow::anyhow!("expected 32 byte array"))?,
        ))
    }
}

impl From<WalletId> for v1alpha1::WalletId {
    fn from(value: WalletId) -> v1alpha1::WalletId {
        v1alpha1::WalletId {
            inner: value.0.to_vec(),
        }
    }
}

impl std::fmt::Debug for WalletId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl std::fmt::Display for WalletId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0,
            bech32str::wallet_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}
