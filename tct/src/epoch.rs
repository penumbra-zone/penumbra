use std::fmt::Display;

use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::{complete, frontier::Forget as _};
use crate::*;

#[path = "block.rs"]
pub(crate) mod block;

#[path = "epoch/error.rs"]
pub mod error;
pub use error::{InsertBlockError, InsertError};

/// A sparse merkle tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536
/// [`Commitment`]s.
///
/// This is one [`Epoch`] in a [`Tree`].
#[derive(Derivative, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Builder {
    index: HashedMap<Commitment, index::within::Epoch>,
    inner: Top<Tier<Item>>,
}

/// A finalized epoch builder, ready to be inserted into an [`Epoch`](super::epoch).
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
pub struct Finalized {
    pub(super) index: HashedMap<Commitment, index::within::Epoch>,
    pub(super) inner: Insert<complete::Top<complete::Tier<complete::Item>>>,
}

impl Default for Finalized {
    fn default() -> Self {
        Builder::default().finalize()
    }
}

impl Finalized {
    /// Get the root hash of this finalized epoch.
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

/// The root hash of an [`Epoch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub(crate) Hash);

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

/// An error occurred when decoding an epoch root from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("could not decode epoch root")]
pub struct RootDecodeError;

impl TryFrom<pb::MerkleRoot> for Root {
    type Error = RootDecodeError;

    fn try_from(root: pb::MerkleRoot) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = (&root.inner[..]).try_into().map_err(|_| RootDecodeError)?;
        let inner = Fq::from_bytes(bytes).map_err(|_| RootDecodeError)?;
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

impl Protobuf<pb::MerkleRoot> for Root {}

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&Fq::from(self.0).to_bytes()))
    }
}

impl Builder {
    /// Create a new empty [`Epoch`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Commitment`] to the most recent [`Block`] of this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if any of:
    ///
    /// - the [`Epoch`] is full, or
    /// - the most recent [`Block`] is full or was inserted by
    ///   [`insert_block_root`](Epoch::insert_block_root).
    pub fn insert(
        &mut self,
        witness: Witness,
        commitment: impl Into<Commitment>,
    ) -> Result<&mut Self, InsertError> {
        let commitment = commitment.into();
        let commitment = match witness {
            Keep => Insert::Keep(commitment),
            Forget => Insert::Hash(Hash::of(commitment)),
        };

        // Get the position of the insertion, if it would succeed
        let position = (self.inner.position().ok_or(InsertError::Full)? as u32).into();

        // Try to insert the commitment into the latest block
        self.inner
            .update(|block| {
                block
                    .insert(commitment.map(Item::new))
                    .map_err(|_| InsertError::BlockFull)?;
                Ok(())
            })
            // If the latest block was finalized already or doesn't exist, create a new block and
            // insert into that block
            .unwrap_or_else(|| {
                self.inner
                    .insert(Insert::Keep(Tier::singleton(commitment.map(Item::new))))
                    .map_err(|_| InsertError::Full)?;
                Ok(())
            })?;

        // Keep track of the position of this just-inserted commitment in the index, if it was
        // slated to be kept
        if let Insert::Keep(commitment) = commitment {
            if let Some(replaced) = self.index.insert(commitment, position) {
                // This case is handled for completeness, but should not happen in
                // practice because commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(self)
    }

    /// Insert a block into this epoch.
    ///
    /// This function can be called on anything that implements `Into<block::Finalized>`; in
    /// particular, on [`block::Builder`], [`block::Finalized`], and [`block::Root`].
    pub fn insert_block(
        &mut self,
        block: impl Into<block::Finalized>,
    ) -> Result<&mut Self, InsertBlockError> {
        let block::Finalized { inner, index } = block.into();

        // Get the block index of the next insertion, if it would succeed
        let mut block = if let Some(position) = self.inner.position() {
            index::within::Epoch::from(position as u32).block
        } else {
            return Err(InsertBlockError(block::Finalized { inner, index }));
        };

        // Determine if the latest-inserted block has yet been finalized (it will implicitly be
        // finalized by the insertion of the block, so we need to know to accurately record the new
        // position)
        let latest_block_finalized = self
            .inner
            .focus()
            .map(|block| block.is_finalized())
            // Epoch is empty or latest block is finalized, so there's nothing to finalize
            .unwrap_or(true);

        // If the latest block was not finalized, then inserting a new block will implicitly
        // finalize the latest block, so the block index to use for indexing new commitments should
        // be one higher than the current block index
        if !latest_block_finalized {
            block.increment();
        }

        // Insert the inner tree of the block into the epoch
        self.inner
            .insert(inner.map(Into::into))
            .expect("inserting a block must succeed when epoch has a position");

        // Add the index of all commitments in the block to the epoch index
        for (c, index::within::Block { commitment }) in index {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(replaced) = self
                .index
                .insert(c, index::within::Epoch { block, commitment })
            {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(self)
    }

    /// Explicitly mark the end of the current block in this epoch, advancing the position to the
    /// next block.
    pub fn end_block(&mut self) -> Result<&mut Self, InsertBlockError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let already_finalized = self
            .inner
            .update(|block| {
                let already_finalized = block.is_finalized();
                block.finalize();
                already_finalized
            })
            // If the entire epoch is empty, the latest block is considered already finalized
            .unwrap_or(true);

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_block(block::Finalized::default())?;
        };

        Ok(self)
    }

    /// Get the root hash of this epoch builder.
    ///
    /// Note that this root hash will differ from the root hash of the finalized epoch.
    pub fn root(&self) -> Root {
        Root(self.inner.hash())
    }

    /// Finalize this epoch builder, returning a finalized epoch and resetting the underlying
    /// builder to the initial empty state.
    pub fn finalize(&mut self) -> Finalized {
        let this = std::mem::take(self);
        let inner = this.inner.finalize();
        let index = this.index;
        Finalized { index, inner }
    }
}
