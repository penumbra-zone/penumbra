use std::fmt::Display;

use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::{frontier::Forget as _, path::Witness as _};
use crate::*;

#[path = "epoch.rs"]
pub(crate) mod epoch;
use epoch::block;

mod proof;
pub use proof::Proof;

pub mod error;
pub use error::{InsertBlockError, InsertEpochError, InsertEpochRootError, InsertError};

/// A sparse merkle tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Commitment`]s.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tree {
    index: HashedMap<Commitment, index::within::Tree>,
    inner: Top<Tier<Tier<Item>>>,
}

/// The root hash of a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub(crate) Hash);

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

/// An error occurred when decoding a tree root from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("could not decode tree root")]
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

/// The index of a [`Commitment`] within a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position(index::within::Tree);

impl Position {
    /// The index of the [`Commitment`] to which this [`Position`] refers within its [`Block`].
    pub fn commitment(&self) -> u16 {
        self.0.commitment.into()
    }

    /// The index of the [`Block`] to which this [`Position`] refers within its [`Epoch`].
    pub fn block(&self) -> u16 {
        self.0.block.into()
    }

    /// The index of the [`Epoch`] to which this [`Position`] refers within its [`Tree`].
    pub fn epoch(&self) -> u16 {
        self.0.epoch.into()
    }
}

impl From<Position> for u64 {
    fn from(position: Position) -> Self {
        position.0.into()
    }
}

impl From<u64> for Position {
    fn from(position: u64) -> Self {
        Position(position.into())
    }
}

impl Height for Tree {
    type Height = <Top<Tier<Tier<Item>>> as Height>::Height;
}

impl Tree {
    /// Create a new empty [`Tree`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the root hash of this [`Tree`].
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

    /// Add a new [`Commitment`] to the most recent [`Block`] of the most recent [`Epoch`] of this
    /// [`Tree`].
    ///
    /// If successful, returns the [`Position`] at which the commitment was inserted.
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if any of:
    ///
    /// - the [`Tree`] is full,
    /// - the most recently inserted [`Epoch`] is full or was inserted by
    /// [`insert_epoch_root`](Tree::insert_epoch_root), or
    /// - the most recently inserted [`Block`] is full or was inserted by
    /// [`insert_block_root`](Tree::insert_block_root).
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
        let position = (self.inner.position().ok_or(InsertError::Full)? as u64).into();

        // Try to insert the commitment into the latest block
        self.inner
            .update(|epoch| {
                epoch
                    .update(|block| {
                        block
                            .insert(commitment.map(Item::new))
                            .map_err(|_| InsertError::BlockFull)?;
                        Ok(())
                    })
                    // If the latest block was finalized already or doesn't exist, create a new block and
                    // insert into that block
                    .unwrap_or_else(|| {
                        epoch
                            .insert(Insert::Keep(Tier::singleton(commitment.map(Item::new))))
                            .map_err(|_| InsertError::EpochFull)?;
                        Ok(())
                    })
            })
            // If the latest epoch was finalized already or doesn't exist, create a new epoch and
            // insert into that epoch
            .unwrap_or_else(|| {
                self.inner
                    .insert(Insert::Keep(Tier::singleton(Insert::Keep(
                        Tier::singleton(commitment.map(Item::new)),
                    ))))
                    .expect("inserting a commitment must succeed because we already checked that the tree is not full");
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

    /// Get a [`Proof`] of inclusion for the commitment at this index in the tree.
    ///
    /// If the index is not witnessed in this tree, return `None`.
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

        let mut forgotten = false;

        if let Some(&within_epoch) = self.index.get(&commitment) {
            // We forgot something
            forgotten = true;
            // Forget the index for this element in the tree
            let forgotten = self.inner.forget(within_epoch);
            debug_assert!(forgotten);
            // Remove this entry from the index
            self.index.remove(&commitment);
        }

        forgotten
    }

    /// Get the position in this [`Tree`] of the given [`Commitment`], if it is currently witnessed.
    pub fn position_of(&self, commitment: impl Into<Commitment>) -> Option<Position> {
        let commitment = commitment.into();
        self.index.get(&commitment).map(|index| Position(*index))
    }

    /// Add a new [`Block`] all at once to the most recently inserted [`Epoch`] of this [`Tree`].
    ///
    /// This function can be called on anything that implements `Into<block::Finalized>`; in
    /// particular, on [`block::Builder`], [`block::Finalized`], and [`block::Root`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockError`] containing the inserted block without adding it to the [`Tree`]
    /// if the [`Tree`] is full, or the most recently inserted [`Epoch`] is full or was inserted by
    /// [`Insert::Hash`].
    pub fn insert_block(
        &mut self,
        block: impl Into<block::Finalized>,
    ) -> Result<&mut Self, InsertBlockError> {
        let block::Finalized { inner, index } = block.into();

        // Get the block index of the next insertion, if it would succeed
        let index::within::Tree {
            epoch, mut block, ..
        } = if let Some(position) = self.inner.position() {
            index::within::Tree::from(position as u64)
        } else {
            return Err(InsertBlockError::Full(block::Finalized { inner, index }));
        };

        // Determine if the latest-inserted block has yet been finalized (it will implicitly be
        // finalized by the insertion of the block, so we need to know to accurately record the new
        // position)
        let latest_block_finalized = self
            .inner
            .focus()
            .and_then(|epoch| epoch.focus().map(|block| block.is_finalized()))
            // Epoch is empty or latest block is complete, so there's nothing to finalize
            .unwrap_or(true);

        // If the latest block was not finalized, then inserting a new block will implicitly
        // finalize the latest block, so the block index to use for indexing new commitments should
        // be one higher than the current block index
        if !latest_block_finalized {
            block.increment();
        }

        // Put the inner tree into an `Option` container so that we can yank it out inside a closure
        // (if the closure is never called, then we can take it *back*)
        let mut inner = Some(inner);

        // Insert the inner tree of the block into the epoch
        match self
            .inner
            .update(|epoch| epoch.insert(inner.take().unwrap().map(Into::into)))
        {
            // Inserting the block into the current epoch succeeded
            Some(Ok(())) => {}
            // Inserting the block into the current epoch failed because the epoch was full but not finalized
            Some(Err(inner)) => {
                // If the insertion failed, map the result back into the input block
                return Err(InsertBlockError::EpochFull(block::Finalized {
                    inner: inner.and_then(|tier| tier.finalize_owned().map(Into::into)),
                    index,
                }));
            }
            None => {
                // Take back the inner thing, which was never inserted, because the closure was
                // never invoked
                let inner = inner.take().unwrap();

                if self.inner.is_full() {
                    // The current epoch was finalized and there is no more room for additional epochs
                    return Err(InsertBlockError::Full(block::Finalized { inner, index }));
                } else {
                    // The current epoch was finalized and there is room to insert a new epoch containing
                    // this block
                    self.inner
                        .insert(Insert::Keep(Tier::singleton(inner.map(Into::into))))
                        .expect("inserting an epoch must succeed when tree has a position");
                }
            }
        }

        // Add the index of all commitments in the block to the global index
        for (c, index::within::Block { commitment }) in index {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(replaced) = self.index.insert(
                c,
                index::within::Tree {
                    epoch,
                    block,
                    commitment,
                },
            ) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(self)
    }

    /// Explicitly mark the end of the current block in this tree, advancing the position to the
    /// next block.
    pub fn end_block(&mut self) -> Result<&mut Self, InsertBlockError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let already_finalized = self
            .inner
            .update(|epoch| {
                epoch.update(|block| {
                    let already_finalized = block.is_finalized();
                    block.finalize();
                    already_finalized
                })
            })
            .flatten()
            // If the entire tree or the latest epoch is empty or finalized, the latest block is
            // considered already finalized
            .unwrap_or(true);

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_block(block::Finalized::default())?;
        };

        Ok(self)
    }

    /// Get the root hash of the most recent [`Block`] in the most recent [`Epoch`] of this
    /// [`Tree`].
    pub fn current_block_root(&self) -> block::Root {
        self.inner
            .focus()
            .and_then(|epoch| {
                let block = epoch.focus()?;
                if block.is_finalized() {
                    None
                } else {
                    Some(block::Root(block.hash()))
                }
            })
            // If there is no latest unfinalized block, we return the hash of the empty unfinalized block
            .unwrap_or_else(|| block::Builder::default().root())
    }

    /// Add a new [`Epoch`] all at once to this [`Tree`].
    ///
    /// This function can be called on anything that implements `Into<epoch::Finalized>`; in
    /// particular, on [`epoch::Builder`], [`epoch::Finalized`], and [`epoch::Root`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertEpochError`] containing the epoch without adding it to the [`Tree`] if
    /// the [`Tree`] is full.
    pub fn insert_epoch(
        &mut self,
        epoch: impl Into<epoch::Finalized>,
    ) -> Result<&mut Self, InsertEpochError> {
        let epoch::Finalized { inner, index } = epoch.into();

        // Get the epoch index of the next insertion, if it would succeed
        let mut epoch = if let Some(position) = self.inner.position() {
            index::within::Tree::from(position as u64).epoch
        } else {
            return Err(InsertEpochError(epoch::Finalized { inner, index }));
        };

        // Determine if the latest-inserted epoch has yet been finalized (it will implicitly be
        // finalized by the insertion of the epoch, so we need to know to accurately record the new
        // position)
        let latest_epoch_finalized = self
            .inner
            .focus()
            .map(|block| block.is_finalized())
            // Tree is empty or latest epoch is finalized, so there's nothing to finalize
            .unwrap_or(true);

        // If the latest epoch was not finalized, then inserting a new epoch will implicitly
        // finalize the latest epoch, so the epoch index to use for indexing new commitments should
        // be one higher than the current epoch index
        if !latest_epoch_finalized {
            epoch.increment();
        }

        // Insert the inner tree of the eooch into the global tree
        match self.inner.insert(inner.map(Into::into)) {
            // Inserting the epoch succeeded
            Ok(()) => {}
            // Inserting the block failed because the epoch was full
            Err(inner) => {
                // If the insertion failed, map the result back into the input block
                return Err(InsertEpochError(epoch::Finalized {
                    inner: inner.and_then(|tier| tier.finalize_owned().map(Into::into)),
                    index,
                }));
            }
        }

        // Add the index of all commitments in the epoch to the global tree index
        for (c, index::within::Epoch { block, commitment }) in index {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(replaced) = self.index.insert(
                c,
                index::within::Tree {
                    epoch,
                    block,
                    commitment,
                },
            ) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(self)
    }

    /// Explicitly mark the end of the current epoch in this tree, advancing the position to the
    /// next epoch.
    pub fn end_epoch(&mut self) -> Result<&mut Self, InsertEpochError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let already_finalized = self
            .inner
            .update(|block| {
                let already_finalized = block.is_finalized();
                block.finalize();
                already_finalized
            })
            // If there is no focused block, the latest block is considered already finalized
            .unwrap_or(true);

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_epoch(epoch::Finalized::default())?;
        };

        Ok(self)
    }

    /// Get the root hash of the most recent [`Epoch`] in this [`Tree`].
    ///
    /// If the [`Tree`] is empty, returns `None`.
    pub fn current_epoch_root(&self) -> epoch::Root {
        self.inner
            .focus()
            .and_then(|epoch| {
                if epoch.is_finalized() {
                    None
                } else {
                    Some(epoch::Root(epoch.hash()))
                }
            })
            // In the case where there is no latest unfinalized epoch, we return the hash of the
            // empty unfinalized epoch
            .unwrap_or_else(|| epoch::Builder::default().root())
    }

    /// The position in this [`Tree`] at which the next [`Commitment`] would be inserted.
    ///
    /// If the [`Tree`] is full, returns `None`.
    ///
    /// The maximum capacity of a [`Tree`] is 281,474,976,710,656 = 65,536 [`Epoch`]s of 65,536
    /// [`Block`]s of 65,536 [`Commitment`]s.
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Tree::witnessed_count).
    pub fn position(&self) -> Option<Position> {
        Some(Position((self.inner.position()? as u64).into()))
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Tree`].
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Tree::position) of the next inserted [`Commitment`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Tree`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
