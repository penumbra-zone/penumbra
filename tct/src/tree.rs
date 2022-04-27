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
pub use error::{
    InsertBlockError, InsertBlockRootError, InsertEpochError, InsertEpochRootError, InsertError,
};

/// A sparse merkle tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Commitment`]s.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tree {
    position: index::within::Tree,
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

/// An error occurred when decoding an eternity root from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("could not decode eternity root")]
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
    ) -> Result<Position, InsertError> {
        todo!()
    }

    /// Get a [`Proof`] of inclusion for the commitment at this index in the eternity.
    ///
    /// If the index is not witnessed in this eternity, return `None`.
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
    ) -> Result<(), InsertBlockError> {
        todo!()
    }

    /// Explicitly mark the end of the current block in this tree, advancing the position to the
    /// next block.
    pub fn end_block(&mut self) -> Result<(), InsertBlockError> {
        todo!()
    }

    /// Get the root hash of the most recent [`Block`] in the most recent [`Epoch`] of this
    /// [`Tree`].
    ///
    /// If the [`Tree`] is empty or the most recent [`Epoch`] was inserted with
    /// [`Tree::insert_epoch_root`], returns `None`.
    pub fn current_block_root(&self) -> Option<block::Root> {
        self.inner.focus().and_then(|epoch| {
            epoch
                .as_ref()
                .keep()?
                .focus()
                .map(|block| block::Root(block.hash()))
        })
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
    ) -> Result<(), InsertEpochError> {
        todo!()
    }

    /// Explicitly mark the end of the current epoch in this tree, advancing the position to the
    /// next epoch.
    pub fn end_epoch(&mut self) -> Result<(), InsertBlockError> {
        todo!()
    }

    /// Get the root hash of the most recent [`Epoch`] in this [`Tree`].
    ///
    /// If the [`Tree`] is empty, returns `None`.
    pub fn current_epoch_root(&self) -> Option<epoch::Root> {
        self.inner.focus().map(|epoch| epoch::Root(epoch.hash()))
    }

    /// The position in this [`Tree`] at which the next [`Commitment`] would be inserted.
    ///
    /// The maximum capacity of a [`Tree`] is 281,474,976,710,656 = 65,536 [`Epoch`]s of 65,536
    /// [`Block`]s of 65,536 [`Commitment`]s.
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Tree::witnessed_count).
    pub fn position(&self) -> Position {
        Position(self.position)
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
