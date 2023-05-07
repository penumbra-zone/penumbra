use std::str::FromStr;

use penumbra_proto::{penumbra::core::transaction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A transaction ID (hash), the Sha256 hash used by Tendermint to identify transactions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::Id", into = "pb::Id")]
pub struct Id(pub [u8; 32]);

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl FromStr for Id {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("invalid transaction ID length"));
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&bytes);
        Ok(Id(id))
    }
}

impl DomainType for Id {
    type Proto = pb::Id;
}

impl From<Id> for pb::Id {
    fn from(id: Id) -> pb::Id {
        pb::Id {
            hash: id.0.to_vec().into(),
        }
    }
}

impl TryFrom<pb::Id> for Id {
    type Error = anyhow::Error;

    fn try_from(proto: pb::Id) -> Result<Id, anyhow::Error> {
        let hash = proto.hash;
        if hash.len() != 32 {
            return Err(anyhow::anyhow!("invalid transaction ID length"));
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        Ok(Id(id))
    }
}
