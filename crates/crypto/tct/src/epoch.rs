use std::fmt::Display;
use std::sync::Arc;

use decaf377::Fq;
use hash_hasher::HashedMap;
use penumbra_proto::{penumbra::crypto::tct::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::error::epoch::*;
use crate::{prelude::*, Witness};

#[path = "block.rs"]
pub(crate) mod block;

/// A sparse merkle tree to witness up to 65,536 blocks, each witnessing up to 65,536
/// [`Commitment`]s.
///
/// This is one epoch in a [`Tree`].
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    index: HashedMap<StateCommitment, index::within::Epoch>,
    inner: Arc<frontier::Top<frontier::Tier<frontier::Item>>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            index: HashedMap::default(),
            inner: Arc::new(frontier::Top::new(frontier::TrackForgotten::No)),
        }
    }
}

/// A finalized epoch builder, ready to be inserted into a [`Tree`].
#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
pub struct Finalized {
    pub(super) index: HashedMap<StateCommitment, index::within::Epoch>,
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

/// The root hash of an epoch.
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

impl From<InsertBlockError> for block::Finalized {
    fn from(error: InsertBlockError) -> Self {
        error.0
    }
}

impl Builder {
    /// Create a new empty [`epoch::Builder`](Builder).
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Commitment`] to the most recent block of this [`epoch::Builder`](Builder).
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if either:
    ///
    /// - the [`epoch::Builder`](Builder) is full, or
    /// - the most recent block is full.
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
        let position = u32::try_from(self.inner.position().ok_or(InsertError::Full)?)
            .expect("position of epoch is never greater than `u32::MAX`")
            .into();

        // Try to insert the commitment into the latest block
        Arc::make_mut(&mut self.inner)
            .update(|block| {
                // Don't insert into a finalized block (this will fail); create a new one instead
                // (below)
                if block.is_finalized() {
                    return None;
                }

                Some(block.insert(item).map_err(|_| InsertError::BlockFull))
            })
            .flatten()
            // If the latest block was finalized already or doesn't exist, create a new block and
            // insert into that block
            .unwrap_or_else(|| {
                Arc::make_mut(&mut self.inner)
                    .insert(frontier::Tier::new(item))
                    .map_err(|_| InsertError::Full)?;
                Ok(())
            })?;

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

    /// Insert a block into this epoch.
    ///
    /// This function can be called on anything that implements `Into<block::Finalized>`; in
    /// particular, on [`block::Builder`], [`block::Finalized`], and [`block::Root`].
    pub fn insert_block(
        &mut self,
        block: impl Into<block::Finalized>,
    ) -> Result<block::Root, InsertBlockError> {
        let block::Finalized { inner, index } = block.into();

        // If the insertion would fail, return an error
        if self.inner.is_full() {
            return Err(InsertBlockError(block::Finalized { inner, index }));
        }

        // Convert the top level inside of the block to a tier that can be slotted into the epoch
        let inner: frontier::Tier<frontier::Item> = match inner {
            Insert::Keep(inner) => inner.into(),
            Insert::Hash(hash) => hash.into(),
        };

        // Finalize the latest block, if it exists and is not yet finalized -- this means that
        // position calculations will be correct, since they will start at the next block
        Arc::make_mut(&mut self.inner).update(|block| block.finalize());

        // Get the block index of the next insertion
        let index::within::Epoch { block, .. } = u32::try_from(
            self.inner
                .position()
                .expect("epoch must have a position because it is not full"),
        )
        .expect("position of epoch is never greater than `u32::MAX`")
        .into();

        // Calculate the root hash of the block being inserted
        let block_root = block::Root(inner.hash());

        // Insert the inner tree of the block into the epoch
        Arc::make_mut(&mut self.inner)
            .insert(inner)
            .expect("inserting a block must succeed because epoch is not full");

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
                let forgotten = Arc::make_mut(&mut self.inner).forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(block_root)
    }

    /// Explicitly mark the end of the current block in this epoch, advancing the position to the
    /// next block.
    pub fn end_block(&mut self) -> Result<block::Root, InsertBlockError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let (already_finalized, finalized_root) = Arc::make_mut(&mut self.inner)
            .update(|tier| {
                let already_finalized = tier.finalize();
                (already_finalized, block::Root(tier.hash()))
            })
            // If the entire epoch is empty, the latest block is considered already finalized
            .unwrap_or((true, block::Finalized::default().root()));

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_block(block::Finalized::default())?;
        };

        Ok(finalized_root)
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

        // This avoids cloning the arc when we have the only reference to it
        let inner = Arc::try_unwrap(this.inner).unwrap_or_else(|arc| (*arc).clone());

        let inner = inner.finalize();
        let index = this.index;
        Finalized { index, inner }
    }
}
