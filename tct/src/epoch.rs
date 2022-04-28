use std::fmt::Display;

use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::frontier::Forget as _;
use crate::*;

#[path = "block.rs"]
pub(crate) mod block;
use block::{Block, BlockMut};

#[path = "epoch/error.rs"]
pub mod error;
pub use error::{InsertBlockError, InsertBlockRootError, InsertError};

/// A sparse merkle tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536
/// [`Commitment`]s.
///
/// This is one [`Epoch`] in a [`Tree`].
#[derive(Derivative, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Epoch {
    pub(super) position: index::within::Epoch,
    pub(super) index: HashedMap<Commitment, index::within::Epoch>,
    pub(super) inner: Tier<Tier<Item>>,
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

/// A mutable reference to an [`Epoch`].
#[derive(Debug)]
pub(super) struct EpochMut<'a> {
    pub(super) commitment: &'a mut index::Commitment,
    pub(super) block: &'a mut index::Block,
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Tier<Item>>,
}

/// A mutable reference to an index from [`Commitment`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Tree`], this index
/// contains all the indices for everything in the tree so far.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum IndexMut<'a> {
    /// An index just for commitments within an epoch.
    Epoch {
        index: &'a mut HashedMap<Commitment, index::within::Epoch>,
    },
    /// An index for commitments within an entire eternity.
    Tree {
        this_epoch: index::Epoch,
        index: &'a mut HashedMap<Commitment, index::within::Tree>,
    },
}

impl Height for Epoch {
    type Height = <Tier<Tier<Item>> as Height>::Height;
}

impl Epoch {
    /// Create a new empty [`Epoch`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an [`EpochMut`] referring to this [`Epoch`].
    pub(super) fn as_mut(&mut self) -> EpochMut {
        EpochMut {
            commitment: &mut self.position.commitment,
            block: &mut self.position.block,
            index: IndexMut::Epoch {
                index: &mut self.index,
            },
            inner: &mut self.inner,
        }
    }

    /// Get the root hash of this [`Epoch`].
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

    /// Add a new [`Commitment`] to the most recent [`Block`] of this [`Epoch`].
    ///
    /// If successful, returns the [`Position`] at which the commitment was inserted.
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
    ) -> Result<(), InsertError> {
        self.as_mut()
            .insert(match witness {
                Keep => Insert::Keep(commitment.into()),
                Forget => Insert::Hash(Hash::of(commitment.into())),
            })
            .map(|replaced| {
                // When inserting into an epoch that's not part of a larger eternity, we should never return
                // further indices to be forgotten, because they should be forgotten internally
                debug_assert!(replaced.is_none());
            })?;

        Ok(())
    }

    /// Add a new [`Block`] all at once to this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockError`] containing the inserted block without adding it to the
    /// [`Epoch`] if the [`Epoch`] is full.
    pub fn insert_block(&mut self, block: Block) -> Result<(), InsertBlockError> {
        self.insert_block_or_root(Insert::Keep(block))
            .map_err(|insert| {
                if let Insert::Keep(block) = insert {
                    InsertBlockError(block)
                } else {
                    unreachable!("failing to insert a block always returns the original block")
                }
            })
    }

    /// Add the root hash of a [`Block`] to this [`Epoch`], without inserting any of the witnessed
    /// commitments in that [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockRootError`] without adding it to the [`Epoch`] if the [`Epoch`] is
    /// full.
    pub fn insert_block_root(
        &mut self,
        block_root: block::Root,
    ) -> Result<(), InsertBlockRootError> {
        self.insert_block_or_root(Insert::Hash(block_root.0))
            .map_err(|insert| {
                if let Insert::Hash(_) = insert {
                    InsertBlockRootError
                } else {
                    unreachable!("failing to insert a block root always returns the original root")
                }
            })
    }

    /// Insert a block or its root (helper function for [`insert_block`] and [`insert_block_root`]).
    fn insert_block_or_root(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        self.as_mut().insert_block_or_root(block).map(|replaced| {
            // When inserting into an epoch that's not part of a larger eternity, we should never return
            // further indices to be forgotten, because they should be forgotten internally
            debug_assert!(replaced.is_empty());
        })
    }
}

impl EpochMut<'_> {
    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the underlying [`Epoch`]: see
    /// [`Epoch::insert`].
    pub fn insert_block_or_root(
        &mut self,
        block: Insert<Block>,
    ) -> Result<Vec<index::within::Tree>, Insert<Block>> {
        // We have a special case when the starting eternity was empty, because then we don't
        // increment the block index
        let was_empty = self.inner.is_empty();

        // All the indices that we've replaced while inserting this block
        let mut replaced_indices = Vec::new();

        // Decompose the block into its components
        let (position, block, block_index) = match block {
            Insert::Hash(hash) => (
                index::within::Block::MAX,
                Insert::Hash(hash),
                Default::default(),
            ),
            Insert::Keep(Block {
                position,
                index,
                inner,
            }) => (position, Insert::Keep(inner), index),
        };

        // Try to insert the block into the tree, and if successful, track the commitment and block
        // indices of each commitment in the inserted block
        if let Err(block) = self.inner.insert(block) {
            Err(block.map(|inner| Block {
                position,
                index: block_index,
                inner,
            }))
        } else {
            // Copy out the commitment index from the just-inserted block
            *self.commitment = position.commitment;

            // Increment the block
            if !was_empty {
                self.block.increment();
            }
            let this_block = *self.block;

            match self.index {
                IndexMut::Epoch { ref mut index } => {
                    for (
                        commitment,
                        index::within::Block {
                            commitment: this_commitment,
                        },
                    ) in block_index.into_iter()
                    {
                        if let Some(replaced) = index.insert(
                            commitment,
                            index::within::Epoch {
                                block: this_block,
                                commitment: this_commitment,
                            },
                        ) {
                            // Immediately forget replaced indices if we are a standalone epoch
                            let forgotten = self.inner.forget(replaced);
                            debug_assert!(forgotten);
                        }
                    }
                }
                IndexMut::Tree {
                    this_epoch,
                    ref mut index,
                } => {
                    for (
                        commitment,
                        index::within::Block {
                            commitment: this_commitment,
                        },
                    ) in block_index.into_iter()
                    {
                        if let Some(index) = index.insert(
                            commitment,
                            index::within::Tree {
                                epoch: this_epoch,
                                block: this_block,
                                commitment: this_commitment,
                            },
                        ) {
                            // If we are part of a larger eternity, collect indices to be forgotten
                            // by the eternity later
                            replaced_indices.push(index)
                        }
                    }
                }
            }

            Ok(replaced_indices)
        }
    }

    /// Insert an commitment into the most recent [`Block`] of this [`Epoch`]: see [`Epoch::insert`].
    pub fn insert(
        &mut self,
        commitment: Insert<Commitment>,
    ) -> Result<Option<index::within::Tree>, InsertError> {
        // If the epoch is empty, we need to create a new block to insert the commitment into
        if self.inner.is_empty()
            && self
                .insert_block_or_root(Insert::Keep(Block::new()))
                .is_err()
        {
            return Err(InsertError::Full);
        }

        match self.update(|block| {
            if let Some(block) = block {
                block.insert(commitment).map_err(|_| InsertError::BlockFull)
            } else {
                Err(InsertError::BlockForgotten)
            }
        }) {
            Err(err) => Err(err),
            Ok(None) => Ok(None),
            Ok(Some(replaced)) => match replaced {
                // If the replaced index was within this epoch, forget it immediately
                block::ReplacedIndex::Epoch(replaced) => {
                    let forgotten = self.inner.forget(replaced);
                    debug_assert!(forgotten);
                    Ok(None)
                }
                // If the replaced index was in a larger eternity, return it to be forgotten above
                block::ReplacedIndex::Tree(replaced) => Ok(Some(replaced)),
            },
        }
    }

    /// Update the most recently inserted [`Block`] via methods on [`BlockMut`], and return the
    /// result of the function.
    pub(super) fn update<T>(&mut self, f: impl FnOnce(Option<&mut BlockMut<'_>>) -> T) -> T {
        let this_block = *self.block;

        let index = match self.index {
            IndexMut::Epoch { ref mut index } => block::IndexMut::Epoch { this_block, index },
            IndexMut::Tree {
                this_epoch,
                ref mut index,
            } => block::IndexMut::Tree {
                this_epoch,
                this_block,
                index,
            },
        };

        self.inner.update(|inner| {
            if let Some(inner) = inner {
                if let Insert::Keep(inner) = inner.as_mut() {
                    f(Some(&mut BlockMut {
                        commitment: self.commitment,
                        inner,
                        index,
                    }))
                } else {
                    f(None)
                }
            } else {
                f(None)
            }
        })
    }
}
