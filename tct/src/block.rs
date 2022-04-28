use std::fmt::Display;

use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::frontier::Forget as _;
use crate::*;

/// A sparse merkle tree to witness up to 65,536 individual [`Commitment`]s.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in a [`Tree`].
#[derive(Derivative, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Block {
    pub(super) position: index::within::Block,
    pub(super) index: HashedMap<Commitment, index::within::Block>,
    pub(super) inner: Tier<Item>,
}

/// The root hash of a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub(crate) Hash);

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

/// An error occurred when decoding a block root from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("could not decode block root")]
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

/// A mutable reference to a [`Block`].
#[derive(Debug)]
pub(in super::super) struct BlockMut<'a> {
    pub(super) commitment: &'a mut index::Commitment,
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Item>,
}

/// A mutable reference to an index from [`Commitment`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Tree`], this index
/// contains all the indices for everything in the tree so far.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum IndexMut<'a> {
    /// An index just for commitments within a block.
    Block {
        index: &'a mut HashedMap<Commitment, index::within::Block>,
    },
    /// An index just for commitments within an epoch.
    Epoch {
        this_block: index::Block,
        index: &'a mut HashedMap<Commitment, index::within::Epoch>,
    },
    /// An index for commitments within an entire eternity.
    Tree {
        this_epoch: index::Epoch,
        this_block: index::Block,
        index: &'a mut HashedMap<Commitment, index::within::Tree>,
    },
}

/// An overwritten index which should be forgotten.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReplacedIndex {
    /// An index from within an epoch.
    Epoch(index::within::Epoch),
    /// An index from within an entire eternity.
    Tree(index::within::Tree),
}

impl Height for Block {
    type Height = <Tier<Item> as Height>::Height;
}

/// When inserting into a block, this error is returned when it is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("block is full")]
#[non_exhaustive]
pub struct InsertError;

impl Block {
    /// Create a new empty [`Block`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a [`BlockMut`] from this [`Block`].
    pub(super) fn as_mut(&mut self) -> BlockMut {
        BlockMut {
            commitment: &mut self.position.commitment,
            index: IndexMut::Block {
                index: &mut self.index,
            },
            inner: &mut self.inner,
        }
    }

    /// Get the root hash of this [`Block`].
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

    /// Add a new [`Commitment`] to this [`Block`].
    ///
    /// If successful, returns the [`Position`] at which the commitment was inserted.
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if the block is full.
    pub fn insert(
        &mut self,
        witness: Witness,
        commitment: impl Into<Commitment>,
    ) -> Result<(), InsertError> {
        self.as_mut()
            .insert(match witness {
                Keep => Insert::Keep(commitment.into()),
                Forget => Insert::Hash(Hash::of(commitment.into())),
            })
            .map(|option|
                // We shouldn't ever be handing back a replaced index here, because the index should
                // be forgotten internally to the method when the block is not owned by a larger structure
                debug_assert!(option.is_none()))
            .map_err(|_| InsertError)?;

        Ok(())
    }
}

impl BlockMut<'_> {
    pub(super) fn insert(
        &mut self,
        commitment: Insert<Commitment>,
    ) -> Result<Option<ReplacedIndex>, Insert<Commitment>> {
        // Try to insert the commitment into the inner tree, and if successful, track the index
        if self.inner.insert(commitment.map(Item::new)).is_err() {
            Err(commitment)
        } else {
            // Increment the position
            let this_commitment = *self.commitment;
            self.commitment.increment();

            // Keep track of the commitment's index in the block, and if applicable, the block's index
            // within its epoch, and if applicable, the epoch's index in the eternity
            if let Insert::Keep(commitment) = commitment {
                match self.index {
                    IndexMut::Block { ref mut index } => {
                        if let Some(replaced) = index.insert(
                            commitment,
                            index::within::Block {
                                commitment: this_commitment,
                            },
                        ) {
                            // This case is handled for completeness, but should not happen in
                            // practice because commitments should be unique
                            self.inner.forget(replaced);
                        }
                        Ok(None)
                    }
                    IndexMut::Epoch {
                        this_block,
                        ref mut index,
                    } => Ok(index
                        .insert(
                            commitment,
                            index::within::Epoch {
                                block: this_block,
                                commitment: this_commitment,
                            },
                        )
                        // We cannot directly forget a replaced index in this case, because it could
                        // refer to a *different* block, so we defer forgetting it to the caller
                        .map(ReplacedIndex::Epoch)),
                    IndexMut::Tree {
                        this_epoch,
                        this_block,
                        ref mut index,
                    } => Ok(index
                        .insert(
                            commitment,
                            index::within::Tree {
                                epoch: this_epoch,
                                block: this_block,
                                commitment: this_commitment,
                            },
                        )
                        // See above about why we defer forgetting the replaced index
                        .map(ReplacedIndex::Tree)),
                }
            } else {
                Ok(None)
            }
        }
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
