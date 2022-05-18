use std::convert::{TryFrom, TryInto};

use ark_ff::{PrimeField, Zero};
use decaf377::{FieldExt, Fq};
use incrementalmerkletree;
pub use incrementalmerkletree::{
    bridgetree::{self, AuthFragment, BridgeTree},
    Altitude, Frontier, Hashable, Position, Recording, Tree,
};
use once_cell::sync::Lazy;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::note;

pub const DEPTH: usize = 32;
pub type NoteCommitmentTree = BridgeTree<note::Commitment, { DEPTH as u8 }>;

/// The domain separator used to hash items into the Merkle tree.
pub static MERKLE_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.merkle.tree").as_bytes())
});

/// An authentication path for a note commitment in the note commitment tree.
///
/// NOTE: this is duplicative of the `Path` typedef below, which is currently
/// used by the transparent proofs.  Deduplicating this should be done when
/// migrating to the TCT, when we'll have to rewrite the transparent proof code anyways.
///
/// TODO: replace this when migrating to TCT.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::AuthPath", into = "pb::AuthPath")]
pub struct AuthPath {
    pub note_commitment: note::Commitment,
    pub position: Position,
    pub path: Vec<note::Commitment>,
}

impl Protobuf<pb::AuthPath> for AuthPath {}

impl TryFrom<pb::AuthPath> for AuthPath {
    type Error = anyhow::Error;

    fn try_from(msg: pb::AuthPath) -> Result<Self, Self::Error> {
        let position = (msg.position as usize).into();
        let note_commitment = msg
            .note_commitment
            .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
            .try_into()?;
        let mut path = Vec::new();
        for entry in msg.path {
            path.push(entry.try_into()?);
        }
        Ok(AuthPath {
            note_commitment,
            position,
            path,
        })
    }
}

impl From<AuthPath> for pb::AuthPath {
    fn from(auth_path: AuthPath) -> Self {
        Self {
            position: u64::from(auth_path.position) as u32,
            note_commitment: Some(auth_path.note_commitment.into()),
            path: auth_path.path.into_iter().map(Into::into).collect(),
        }
    }
}

// Return value from `Tree::authentication_path(value: &note::Commitment)`
pub type Path = (Position, Vec<note::Commitment>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
pub struct Root(pub Fq);

impl Protobuf<pb::MerkleRoot> for Root {}

impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0.to_bytes()))
    }
}

impl std::fmt::Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("merkle::Root")
            .field(&hex::encode(&self.0.to_bytes()))
            .finish()
    }
}

impl TryFrom<pb::MerkleRoot> for Root {
    type Error = anyhow::Error;

    fn try_from(root: pb::MerkleRoot) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = (&root.inner[..]).try_into()?;

        let inner = Fq::from_bytes(bytes)?;

        Ok(Root(inner))
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

        Ok(Root(inner))
    }
}

impl Root {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

pub trait TreeExt {
    fn root2(&self) -> Root;

    fn auth_path(&self, note_commitment: note::Commitment) -> Option<AuthPath>;
}

impl<T> TreeExt for T
where
    T: Tree<note::Commitment>,
{
    fn root2(&self) -> Root {
        Root(self.root().0)
    }

    fn auth_path(&self, note_commitment: note::Commitment) -> Option<AuthPath> {
        self.authentication_path(&note_commitment)
            .map(|(position, path)| AuthPath {
                note_commitment,
                position,
                path,
            })
    }
}

impl Hashable for note::Commitment {
    fn empty_leaf() -> Self {
        note::Commitment(Fq::zero())
    }

    fn combine(level: Altitude, a: &Self, b: &Self) -> Self {
        // extend to build domain sep
        let level_fq: Fq = u8::from(level).into();
        let level_domain_sep: Fq = *MERKLE_DOMAIN_SEP + level_fq;
        note::Commitment(poseidon377::hash_2(&level_domain_sep, (a.0, b.0)))
    }
}
