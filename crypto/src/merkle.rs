use std::convert::{TryFrom, TryInto};

use decaf377::{FieldExt, Fq};
use penumbra_proto::{crypto as pb, Protobuf};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

pub type NoteCommitmentTree = tct::Eternity;

// Return value from `Tree::authentication_path(value: &note::Commitment)`
pub type Path = [[tct::Hash; 3]; 24];

pub use tct::{Eternity as Tree, Forget, Keep, Position, Proof};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
pub struct Root(pub tct::Root);

impl From<tct::Root> for Root {
    fn from(root: tct::Root) -> Self {
        Self(root)
    }
}

impl From<Root> for tct::Root {
    fn from(root: Root) -> Self {
        root.0
    }
}

impl Protobuf<pb::MerkleRoot> for Root {}

impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&Fq::from(self.0).to_bytes()))
    }
}

impl std::fmt::Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("merkle::Root")
            .field(&hex::encode(&Fq::from(self.0).to_bytes()))
            .finish()
    }
}

impl TryFrom<pb::MerkleRoot> for Root {
    type Error = anyhow::Error;

    fn try_from(root: pb::MerkleRoot) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = (&root.inner[..]).try_into()?;

        let inner = Fq::from_bytes(bytes)?;

        Ok(Root(inner.into()))
    }
}

impl From<Root> for pb::MerkleRoot {
    fn from(root: Root) -> Self {
        Self {
            inner: root.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<&[u8]> for Root {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into()?;

        let inner = Fq::from_bytes(bytes)?;

        Ok(Root(inner.into()))
    }
}

impl Root {
    pub fn to_bytes(&self) -> [u8; 32] {
        Fq::from(self.0).to_bytes()
    }
}
