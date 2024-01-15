use std::str::FromStr;

use penumbra_proto::{penumbra::core::txhash::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A transaction ID (hash), the Sha256 hash used by Tendermint to identify transactions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::TransactionId", into = "pb::TransactionId")]
pub struct TransactionId(pub [u8; 32]);

impl AsRef<[u8]> for TransactionId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl FromStr for TransactionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            anyhow::bail!("invalid transaction ID length");
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&bytes);
        Ok(TransactionId(id))
    }
}

impl DomainType for TransactionId {
    type Proto = pb::TransactionId;
}

impl From<TransactionId> for pb::TransactionId {
    fn from(id: TransactionId) -> pb::TransactionId {
        pb::TransactionId {
            inner: id.0.to_vec(),
        }
    }
}

impl TryFrom<pb::TransactionId> for TransactionId {
    type Error = anyhow::Error;

    fn try_from(proto: pb::TransactionId) -> anyhow::Result<TransactionId> {
        let hash = proto.inner;
        if hash.len() != 32 {
            anyhow::bail!("invalid transaction ID length");
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        Ok(TransactionId(id))
    }
}
