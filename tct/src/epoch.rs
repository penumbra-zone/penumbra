use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::crypto as pb;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::{active::Forget as _, path::Witness as _};
use crate::*;

#[path = "block.rs"]
pub(crate) mod block;
use block::{Block, BlockMut};

#[path = "epoch/proof.rs"]
mod proof;
pub use proof::Proof;

#[path = "epoch/error.rs"]
pub mod error;
pub use error::{InsertBlockError, InsertBlockRootError, InsertError};

/// A sparse merkle tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536
/// [`Commitment`]s.
///
/// This is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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
        Ok(Root(Hash(inner)))
    }
}

impl From<Root> for pb::MerkleRoot {
    fn from(root: Root) -> Self {
        Self {
            inner: root.0 .0.to_bytes().to_vec(),
        }
    }
}

/// The index of a [`Commitment`] within an [`Epoch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position(u32);

impl From<Position> for u32 {
    fn from(position: Position) -> Self {
        position.0
    }
}

impl From<u32> for Position {
    fn from(position: u32) -> Self {
        Position(position)
    }
}

/// A mutable reference to an [`Epoch`].
#[derive(Debug, PartialEq, Eq)]
pub(super) struct EpochMut<'a> {
    pub(super) commitment: &'a mut index::Commitment,
    pub(super) block: &'a mut index::Block,
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Tier<Item>>,
}

/// A mutable reference to an index from [`Commitment`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Eternity`], this index
/// contains all the indices for everything in the tree so far.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum IndexMut<'a> {
    /// An index just for commitments within an epoch.
    Epoch {
        index: &'a mut HashedMap<Commitment, index::within::Epoch>,
    },
    /// An index for commitments within an entire eternity.
    Eternity {
        this_epoch: index::Epoch,
        index: &'a mut HashedMap<Commitment, index::within::Eternity>,
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
    ) -> Result<Position, InsertError> {
        // The position at which we will insert the commitment
        let position = self.position();

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

        Ok(position)
    }

    /// Get a [`Proof`] of inclusion for the commitment at this index in the epoch.
    ///
    /// If the index is not witnessed in this epoch, return `None`.
    pub fn witness(&self, commitment: impl Into<Commitment>) -> Option<Proof> {
        let commitment = commitment.into();

        let index = *self.index.get(&commitment)?;

        let (auth_path, leaf) = match self.inner.witness(index) {
            Some(witness) => witness,
            None => panic!(
                "commitment `{:?}` indexed with position `{:?}` must be witnessed",
                commitment, index
            ),
        };
        debug_assert_eq!(leaf, Hash::of(commitment));

        Some(Proof(crate::proof::Proof {
            position: index.into(),
            auth_path,
            leaf: commitment,
        }))
    }

    /// Forget about the witness for the given [`Commitment`].
    ///
    /// Returns `true` if the commitment was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, commitment: impl Into<Commitment>) -> bool {
        let commitment = commitment.into();
        self.as_mut().forget(commitment)
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

    /// Get the root hash of the most recent [`Block`] in this [`Epoch`].
    ///
    /// If the [`Epoch`] is empty, returns `None`.
    pub fn current_block_root(&self) -> Option<block::Root> {
        self.inner.focus().map(|block| block::Root(block.hash()))
    }

    /// The position in this [`Epoch`] at which the next [`Commitment`] would be inserted.
    ///
    /// The maximum capacity of an [`Epoch`] is 4,294,967,296, = 65,536 [`Block`]s of 65,536
    /// [`Commitment`]s.
    ///
    /// Note that [`forget`](Epoch::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Epoch::witnessed_count).
    pub fn position(&self) -> Position {
        Position(self.position.into())
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Epoch`].
    ///
    /// Note that [`forget`](Epoch::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Epoch::position) of the next inserted [`Commitment`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Epoch`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl EpochMut<'_> {
    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the underlying [`Epoch`]: see
    /// [`Epoch::insert`].
    pub fn insert_block_or_root(
        &mut self,
        block: Insert<Block>,
    ) -> Result<Vec<index::within::Eternity>, Insert<Block>> {
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
                IndexMut::Eternity {
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
                            index::within::Eternity {
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
    ) -> Result<Option<index::within::Eternity>, InsertError> {
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
                block::ReplacedIndex::Eternity(replaced) => Ok(Some(replaced)),
            },
        }
    }

    /// Forget the witness of the given commitment, if it was witnessed: see [`Epoch::forget`].
    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;

        match self.index {
            IndexMut::Epoch { ref mut index } => {
                if let Some(&within_epoch) = index.get(&commitment) {
                    // We forgot something
                    forgotten = true;
                    // Forget the index for this element in the tree
                    let forgotten = self.inner.forget(within_epoch);
                    debug_assert!(forgotten);
                    // Remove this entry from the index
                    index.remove(&commitment);
                }
            }
            IndexMut::Eternity {
                this_epoch,
                ref mut index,
            } => {
                if let Some(&within_eternity) = index.get(&commitment) {
                    // Only forget this index if it belongs to the current epoch
                    if within_eternity.epoch == this_epoch {
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

    /// Update the most recently inserted [`Block`] via methods on [`BlockMut`], and return the
    /// result of the function.
    pub(super) fn update<T>(&mut self, f: impl FnOnce(Option<&mut BlockMut<'_>>) -> T) -> T {
        let this_block = *self.block;

        let index = match self.index {
            IndexMut::Epoch { ref mut index } => block::IndexMut::Epoch { this_block, index },
            IndexMut::Eternity {
                this_epoch,
                ref mut index,
            } => block::IndexMut::Eternity {
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

#[cfg(feature = "sqlx")]
mod sqlx_impls {
    use sqlx::{Database, Decode, Encode, Postgres, Type};

    use super::*;

    impl<'r> Decode<'r, Postgres> for Root {
        fn decode(
            value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            Ok(Root(Hash::decode(value)?))
        }
    }

    impl<'q> Encode<'q, Postgres> for Root {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            self.0.encode_by_ref(buf)
        }
    }

    impl Type<Postgres> for Root {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            <[u8] as Type<Postgres>>::type_info()
        }
    }
}
