use std::str::FromStr;

use penumbra_proto::{penumbra::core::transaction::v1alpha1 as pb, DomainType, TypeUrl};
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

impl TypeUrl for Id {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.Id";
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// Ensure that a transaction identifier, as emitted by `pcli`,
    /// can be transmuted into a proto type, then converted back
    /// into a Rust DomainType and be identical.
    fn tx_hash_proto_roundtrip() -> anyhow::Result<()> {
        // Hex-encoded string of tx id taken from `pcli tx send ...` invocation.
        let s1 = "f065a14cb75a29806969755916bc338549c4841a66060b404f557c5c6ea03aa0";
        let tx1: Id = s1.parse()?;
        let txp: pb::Id = tx1.try_into()?;
        let tx2: Id = txp.try_into()?;
        let s2: String = format!("{}", tx2);
        // We expect the equality assertion to fail, to confirm the Lovecraftian hex-encoding bug.
        // But fail it does NOT. Which, in this case, makes me sad.
        assert_eq!(s1, s2);
        Ok(())
    }
}
