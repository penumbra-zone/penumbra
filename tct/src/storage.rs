//! Incremental serialization and non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Range,
    pin::Pin,
};

use futures::Stream;

use crate::prelude::*;

pub(crate) mod deserialize;
pub(crate) mod serialize;

pub mod in_memory;
pub use in_memory::InMemory;

/// A stored position for the tree: either the position of the tree, or a marker indicating that it
/// is full, and therefore does not have a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StoredPosition {
    /// The tree has the given position.
    Position(Position),
    /// The tree is full.
    Full,
}

impl Default for StoredPosition {
    fn default() -> Self {
        StoredPosition::Position(Position::default())
    }
}

impl From<StoredPosition> for Option<Position> {
    fn from(stored: StoredPosition) -> Self {
        match stored {
            StoredPosition::Position(position) => Some(position),
            StoredPosition::Full => None,
        }
    }
}

impl From<Option<Position>> for StoredPosition {
    fn from(position: Option<Position>) -> Self {
        match position {
            Some(position) => StoredPosition::Position(position),
            None => StoredPosition::Full,
        }
    }
}

/// An `async` storage backend capable of reading stored [`struct@Hash`]es and [`Commitment`]s as
/// well as storing the current [`Position`].
#[async_trait]
pub trait AsyncRead {
    /// The error returned when something goes wrong in a request.
    type Error;

    /// Fetch the current position stored.
    async fn position(&mut self) -> Result<StoredPosition, Self::Error>;

    /// Fetch the current forgotten version.
    async fn forgotten(&mut self) -> Result<Forgotten, Self::Error>;

    /// Fetch the hash at the given position and height, if it exists.
    async fn hash(&mut self, position: Position, height: u8) -> Result<Option<Hash>, Self::Error>;

    /// Get the full list of all internal hashes stored, indexed by position and height.
    #[allow(clippy::type_complexity)]
    fn hashes(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, u8, Hash), Self::Error>> + Send + '_>>;

    /// Fetch the commitment at the given position, if it exists.
    async fn commitment(&mut self, position: Position) -> Result<Option<Commitment>, Self::Error>;

    /// Get the full list of all commitments stored, indexed by position.
    #[allow(clippy::type_complexity)]
    fn commitments(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, Commitment), Self::Error>> + Send + '_>>;
}

/// An `async` storage backend capable of writing [`struct@Hash`]es and [`Commitment`]s, and
/// garbage-collecting those which have been forgotten.
#[async_trait]
pub trait AsyncWrite: AsyncRead {
    /// Write a single hash into storage.
    ///
    /// Backends are only *required* to persist hashes marked as `essential`. They may choose to
    /// persist other hashes, and the choice of which non-essential hashes to persist is
    /// unconstrained. However, choosing not to persist non-essential hashes imposes computational
    /// overhead upon deserialization.
    async fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
        essential: bool,
    ) -> Result<(), Self::Error>;

    /// Write a single commitment into storage.
    ///
    /// This should return an error if a commitment is already present at that location; no
    /// location's value should ever be overwritten.
    async fn add_commitment(
        &mut self,
        position: Position,
        commitment: Commitment,
    ) -> Result<(), Self::Error>;

    /// Delete every stored [`struct@Hash`] whose height is less than `below_height` and whose
    /// position is within the half-open [`Range`] of `positions`, as well as every [`Commitment`]
    /// whose position is within the range.
    async fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error>;

    /// Set the stored position of the tree.
    ///
    /// This should return an error if the position goes backwards.
    async fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error>;

    /// Set the forgotten version of the tree.
    ///
    /// This should return an error if the version goes backwards.
    async fn set_forgotten(&mut self, forgotten: Forgotten) -> Result<(), Self::Error>;
}

/// A synchronous storage backend capable of reading stored [`struct@Hash`]es and [`Commitment`]s as
/// well as storing the current [`Position`].
pub trait Read {
    /// The error returned when something goes wrong in a request.
    type Error;

    /// Fetch the current position stored.
    fn position(&mut self) -> Result<StoredPosition, Self::Error>;

    /// Fetch the current forgotten version.
    fn forgotten(&mut self) -> Result<Forgotten, Self::Error>;

    /// Fetch a specific hash at the given position and height, if it exists.
    fn hash(&mut self, position: Position, height: u8) -> Result<Option<Hash>, Self::Error>;

    /// Get the full list of all internal hashes stored, indexed by position and height.
    #[allow(clippy::type_complexity)]
    fn hashes(
        &mut self,
    ) -> Box<dyn Iterator<Item = Result<(Position, u8, Hash), Self::Error>> + Send + '_>;

    /// Fetch a specific commitment at the given position, if it exists.
    fn commitment(&mut self, position: Position) -> Result<Option<Commitment>, Self::Error>;

    /// Get the full list of all commitments stored, indexed by position.
    #[allow(clippy::type_complexity)]
    fn commitments(
        &mut self,
    ) -> Box<dyn Iterator<Item = Result<(Position, Commitment), Self::Error>> + Send + '_>;
}

/// A synchronous storage backend capable of writing [`struct@Hash`]es and [`Commitment`]s, and
/// garbage-collecting those which have been forgotten.
pub trait Write: Read {
    /// Write a single hash into storage.
    ///
    /// Backends are only *required* to persist hashes marked as `essential`. They may choose to
    /// persist other hashes, and the choice of which non-essential hashes to persist is
    /// unconstrained. However, choosing not to persist non-essential hashes imposes computational
    /// overhead upon deserialization.
    fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
        essential: bool,
    ) -> Result<(), Self::Error>;

    /// Write a single commitment into storage.
    ///
    /// This should return an error if a commitment is already present at that location; no
    /// location's value should ever be overwritten.
    fn add_commitment(
        &mut self,
        position: Position,
        commitment: Commitment,
    ) -> Result<(), Self::Error>;

    /// Delete every stored [`struct@Hash`] whose height is less than `below_height` and whose
    /// position is within the half-open [`Range`] of `positions`, as well as every [`Commitment`]
    /// whose position is within the range.
    fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error>;

    /// Set the stored position of the tree.
    ///
    /// This should return an error if the position goes backwards.
    fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error>;

    /// Set the forgotten version of the tree.
    ///
    /// This should return an error if the version goes backwards.
    fn set_forgotten(&mut self, forgotten: Forgotten) -> Result<(), Self::Error>;
}

/// A single update to the underlying storage, as a data type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    /// Set the position of the tree.
    SetPosition(StoredPosition),
    /// Set the forgotten version of the tree.
    SetForgotten(Forgotten),
    /// Add a commitment to the tree.
    StoreCommitment(StoreCommitment),
    /// Add a hash to the tree.
    StoreHash(StoreHash),
    /// Delete a range of hashes and commitments from the tree.
    DeleteRange(DeleteRange),
}

/// An update to the underlying storage that constitutes storing a single hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreHash {
    /// The position of the hash.
    pub position: Position,
    /// The height of the hash.
    pub height: u8,
    /// The hash itself.
    pub hash: Hash,
    /// Whether the hash is essential to store, or can be dropped at the discretion of the storage backend.
    pub essential: bool,
}

/// An update to the underlying storage that constitutes storing a single commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreCommitment {
    /// The position of the commitment.
    pub position: Position,
    /// The commitment itself.
    pub commitment: Commitment,
}

/// An update to the underlying storage that constitutes deleting a range of hashes and commitments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRange {
    /// The height strictly below which hashes should be deleted.
    pub below_height: u8,
    /// The half-open range of positions within which hashes and commitments should be deleted.
    pub positions: Range<Position>,
}

/// A collection of updates to the underlying storage.
///
/// Note that this is both `FromIterator<Update>` and `Iterator<Item = Update>`, so you can
/// [`.collect()](Iterator::collect) an `impl Iterator<Item = Update>` into this type, and you can
/// also iterate over its contained updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Updates {
    /// The new position to set, if any.
    pub set_position: Option<StoredPosition>,
    /// The new forgotten version to set, if any.
    pub set_forgotten: Option<Forgotten>,
    /// The new commitments to store.
    pub store_commitments: Vec<StoreCommitment>,
    /// The new hashes to store.
    pub store_hashes: Vec<StoreHash>,
    /// The ranges of hashes and commitments to delete.
    pub delete_ranges: Vec<DeleteRange>,
}

impl FromIterator<Update> for Updates {
    fn from_iter<I: IntoIterator<Item = Update>>(iter: I) -> Self {
        let mut updates = Updates::default();
        for update in iter {
            match update {
                Update::SetPosition(position) => updates.set_position = Some(position),
                Update::SetForgotten(forgotten) => updates.set_forgotten = Some(forgotten),
                Update::StoreCommitment(commitment) => updates.store_commitments.push(commitment),
                Update::StoreHash(hash) => updates.store_hashes.push(hash),
                Update::DeleteRange(range) => updates.delete_ranges.push(range),
            }
        }
        updates
    }
}

impl Iterator for Updates {
    type Item = Update;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(position) = self.set_position.take() {
            return Some(Update::SetPosition(position));
        }
        if let Some(forgotten) = self.set_forgotten.take() {
            return Some(Update::SetForgotten(forgotten));
        }
        if let Some(commitment) = self.store_commitments.pop() {
            return Some(Update::StoreCommitment(commitment));
        }
        if let Some(hash) = self.store_hashes.pop() {
            return Some(Update::StoreHash(hash));
        }
        if let Some(range) = self.delete_ranges.pop() {
            return Some(Update::DeleteRange(range));
        }
        None
    }
}
