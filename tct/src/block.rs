use decaf377::Fq;
use hash_hasher::HashedMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::{active::Forget as _, path::Witness as _};
use crate::*;

#[path = "block/proof.rs"]
mod proof;
pub use proof::{Proof, VerifyError};

/// A sparse merkle tree to witness up to 65,536 individual [`Commitment`]s.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Block {
    pub(super) index: HashedMap<Commitment, index::within::Block>,
    pub(super) inner: Tier<Item>,
}

/// The root hash of a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Root(pub(in super::super) Hash);

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

impl From<Fq> for Root {
    fn from(root: Fq) -> Self {
        Root(Hash(root))
    }
}

/// The index of a [`Commitment`] within a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position(u16);

impl From<Position> for u16 {
    fn from(position: Position) -> Self {
        position.0
    }
}

impl From<u16> for Position {
    fn from(position: u16) -> Self {
        Position(position)
    }
}

/// A mutable reference to a [`Block`].
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct BlockMut<'a> {
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Item>,
}

/// A mutable reference to an index from [`Commitment`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Eternity`], this index
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
    Eternity {
        this_epoch: index::Epoch,
        this_block: index::Block,
        index: &'a mut HashedMap<Commitment, index::within::Eternity>,
    },
}

/// An overwritten index which should be forgotten.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReplacedIndex {
    /// An index from within an epoch.
    Epoch(index::within::Epoch),
    /// An index from within an entire eternity.
    Eternity(index::within::Eternity),
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
    /// # Errors
    ///
    /// Returns [`InsertError`] if the block is full.
    pub fn insert(&mut self, witness: Witness, commitment: Commitment) -> Result<(), InsertError> {
        self.as_mut()
            .insert(match witness {
                Keep => Insert::Keep(commitment),
                Forget => Insert::Hash(Hash::of(commitment)),
            })
            .map(|option|
                // We shouldn't ever be handing back a replaced index here, because the index should
                // be forgotten internally to the method when the block is not owned by a larger structure
                debug_assert!(option.is_none()))
            .map_err(|_| InsertError)
    }

    /// Get a [`Proof`] of inclusion for this commitment in the block.
    ///
    /// If the index is not witnessed in this block, return `None`.
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let index = *self.index.get(&commitment)?;

        let (auth_path, leaf) = self.inner.witness(index)?;
        debug_assert_eq!(leaf, Hash::of(commitment));

        Some(Proof(crate::proof::Proof {
            index: index.into(),
            auth_path,
            leaf: commitment,
        }))
    }

    /// Forget the witness of the given commitment, if it was witnessed.
    ///
    /// Returns `true` if the commitment was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, commitment: Commitment) -> bool {
        self.as_mut().forget(commitment)
    }

    /// The position in this [`Block`] at which the next [`Commitment`] would be inserted.
    ///
    /// The maximum capacity of an [`Block`] is 65,536 [`Commitment`]s.
    ///
    /// Note that [`forget`](Block::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Block::witnessed_count).
    pub fn position(&self) -> Position {
        Position(self.inner.position())
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Block`].
    ///
    /// Note that [`forget`](Block::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Block::position) of the next inserted [`Commitment`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether the underlying [`Block`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl BlockMut<'_> {
    pub(super) fn insert(
        &mut self,
        commitment: Insert<Commitment>,
    ) -> Result<Option<ReplacedIndex>, Insert<Commitment>> {
        // If we successfully insert this commitment, here's what its index in the block will be:
        let this_commitment: index::Commitment = self.inner.position().into();

        // Try to insert the commitment into the inner tree, and if successful, track the index
        if self.inner.insert(commitment.map(Item::new)).is_err() {
            Err(commitment)
        } else {
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
                        .map(ReplacedIndex::Epoch)),
                    IndexMut::Eternity {
                        this_epoch,
                        this_block,
                        ref mut index,
                    } => Ok(index
                        .insert(
                            commitment,
                            index::within::Eternity {
                                epoch: this_epoch,
                                block: this_block,
                                commitment: this_commitment,
                            },
                        )
                        .map(ReplacedIndex::Eternity)),
                }
            } else {
                Ok(None)
            }
        }
    }

    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;

        match self.index {
            IndexMut::Block { ref mut index } => {
                if let Some(&within_block) = index.get(&commitment) {
                    // We forgot something
                    forgotten = true;
                    // Forget the index for this element in the tree
                    let forgotten = self.inner.forget(within_block);
                    debug_assert!(forgotten);
                    // Remove this entry from the index
                    index.remove(&commitment);
                }
            }
            IndexMut::Epoch {
                this_block,
                ref mut index,
            } => {
                if let Some(&within_epoch) = index.get(&commitment) {
                    // Only forget this index if it belongs to the current block
                    if within_epoch.block == this_block {
                        // We forgot something
                        forgotten = true;
                        // Forget the index for this element in the tree
                        let forgotten = self.inner.forget(within_epoch);
                        debug_assert!(forgotten);
                        // Remove this entry from the index
                        index.remove(&commitment);
                    }
                }
            }
            IndexMut::Eternity {
                this_epoch,
                this_block,
                ref mut index,
            } => {
                if let Some(&within_eternity) = index.get(&commitment) {
                    // Only forget this index if it belongs to the current block and that block
                    // belongs to the current epoch
                    if within_eternity.block == this_block && within_eternity.epoch == this_epoch {
                        // We forgot something
                        forgotten = true;
                        // Forget the index for this element in the tree
                        let forgotten = self.inner.forget(within_eternity);
                        debug_assert!(forgotten);
                        // Remove this entry from the index
                        index.remove(&commitment);
                    }
                }
            }
        }

        forgotten
    }
}
