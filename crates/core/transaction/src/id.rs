use bytes::Bytes;
use penumbra_proto::{penumbra::core::transaction::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

    /// Generates an Id from a hex-encoded string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        tracing::debug!(
            "parsing transaction ID string: {:?} length: {:?}",
            s,
            s.len()
        );

        let bytes = hex::decode(s)?;

        tracing::debug!(
            "hex-decoded transaction ID string bytes: {:?} length: {:?}",
            bytes,
            bytes.len()
        );

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
        tracing::debug!("From<Id> Id: {:?} length: {:?}", &id.0, &id.0.len());
        let hash: Bytes = id.0.to_vec().into();
        tracing::debug!("From<Id> pb::Id.hash {:?} length: {:?}", &hash, &hash.len());
        pb::Id { hash }
    }
}

impl TryFrom<pb::Id> for Id {
    type Error = anyhow::Error;

    fn try_from(proto: pb::Id) -> Result<Id, anyhow::Error> {
        let hash = proto.hash;
        tracing::debug!(
            "TryFrom<pb::Id> pb::Id.hash: {:?} length: {:?}",
            &hash,
            &hash.len()
        );
        // Doing a length check on the proto.hash bytes can fail because the protobuf Bytes field will not always reflect the converted length
        if hash.len() != 32 {
            return Err(anyhow::anyhow!("invalid transaction ID length"));
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        tracing::debug!("TryFrom<pb::Id> Id: {:?} length: {:?}", &id, &id.len());

        Ok(Id(id))
    }
}
