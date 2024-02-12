use std::fmt::Display;
use std::sync::Arc;

use decaf377::Fq;
use hash_hasher::HashedMap;
use penumbra_proto::{penumbra::crypto::tct::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::error::block::*;
use crate::{prelude::*, Witness};

/// A sparse merkle tree to witness up to 65,536 individual [`Commitment`]s.
///
/// This is one block in an [`epoch`](crate::builder::epoch), which is one epoch in a [`Tree`].
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    index: HashedMap<StateCommitment, index::within::Block>,
    inner: Arc<frontier::Top<Item>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            index: HashedMap::default(),
            inner: Arc::new(frontier::Top::new(frontier::TrackForgotten::No)),
        }
    }
}

/// A finalized block builder, ready to be inserted into an [`epoch::Builder`](super::Builder) or a
/// [`Tree`].
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
pub struct Finalized {
    pub(in super::super) index: HashedMap<StateCommitment, index::within::Block>,
    pub(in super::super) inner: Insert<complete::Top<complete::Item>>,
}

impl Default for Finalized {
    fn default() -> Self {
        Builder::default().finalize()
    }
}

impl Finalized {
    /// Get the root hash of this finalized block.
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    pub fn root(&self) -> Root {
        Root(self.inner.hash())
    }
}

impl From<Root> for Finalized {
    fn from(root: Root) -> Self {
        Self {
            index: HashedMap::default(),
            inner: Insert::Hash(root.0),
        }
    }
}

impl From<Builder> for Finalized {
    fn from(mut builder: Builder) -> Self {
        builder.finalize()
    }
}

/// The root hash of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub Hash);

impl Root {
    /// Check if this is the root of an empty finalized block.
    pub fn is_empty_finalized(&self) -> bool {
        self.0 == Hash::one()
    }

    /// Check if this is the root of an empty unfinalized block.
    pub fn is_empty_unfinalized(&self) -> bool {
        self.0 == Hash::zero()
    }
}

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

impl TryFrom<pb::MerkleRoot> for Root {
    type Error = RootDecodeError;

    fn try_from(root: pb::MerkleRoot) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = (&root.inner[..]).try_into().map_err(|_| RootDecodeError)?;
        let inner = Fq::from_bytes_checked(&bytes).map_err(|_| RootDecodeError)?;
        Ok(Root(Hash::new(inner)))
    }
}

impl From<Root> for pb::MerkleRoot {
    fn from(root: Root) -> Self {
        Self {
            inner: Fq::from(root.0).to_bytes().to_vec(),
        }
    }
}

impl DomainType for Root {
    type Proto = pb::MerkleRoot;
}

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(Fq::from(self.0).to_bytes()))
    }
}

impl Builder {
    /// Create a new empty [`block::Builder`](Builder).
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Commitment`] to this [`block::Builder`](Builder).
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if the block is full.
    pub fn insert(
        &mut self,
        witness: Witness,
        commitment: StateCommitment,
    ) -> Result<(), InsertError> {
        let item = match witness {
            Witness::Keep => commitment.into(),
            Witness::Forget => Hash::of(commitment).into(),
        };

        // Get the position of the insertion, if it would succeed
        let position = u16::try_from(self.inner.position().ok_or(InsertError)?)
            .expect("position of block is never greater than `u16::MAX`")
            .into();

        // Insert the commitment into the inner tree
        Arc::make_mut(&mut self.inner)
            .insert(item)
            .expect("inserting a commitment must succeed when block has a position");

        // Keep track of the position of this just-inserted commitment in the index, if it was
        // slated to be kept
        if let Witness::Keep = witness {
            if let Some(replaced) = self.index.insert(commitment, position) {
                // This case is handled for completeness, but should not happen in
                // practice because commitments should be unique
                let forgotten = Arc::make_mut(&mut self.inner).forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(())
    }

    /// Get the root hash of this block builder.
    ///
    /// Note that this root hash will differ from the root hash of the finalized block.
    pub fn root(&self) -> Root {
        Root(self.inner.hash())
    }

    /// Finalize this block builder returning a finalized block and resetting the underlying builder
    /// to the initial empty state.
    pub fn finalize(&mut self) -> Finalized {
        let this = std::mem::take(self);

        // This avoids cloning the arc when we have the only reference to it
        let inner = Arc::try_unwrap(this.inner).unwrap_or_else(|arc| (*arc).clone());

        let inner = inner.finalize();
        let index = this.index;
        Finalized { index, inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_error_sync_send() {
        static_assertions::assert_impl_all!(InsertError: Sync, Send);
    }
}
