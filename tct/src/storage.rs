//! Incremental serialization and non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Range,
    pin::Pin,
};

use futures::{stream, Stream};

use crate::prelude::*;

pub(crate) mod deserialize;
pub(crate) use deserialize::from_reader;

pub mod in_memory;
pub use in_memory::InMemory;

pub(crate) mod serialize;
pub(crate) use serialize::to_writer;

/// A stored position for the tree: either the position of the tree, or a marker indicating that it
/// is full, and therefore does not have a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// A storage backend capable of reading stored [`struct@Hash`]es and [`Commitment`]s as well as
/// storing the current [`Position`].
#[async_trait]
pub trait Read {
    /// The error returned when something goes wrong in a request.
    type Error;

    /// Fetch the current position stored.
    async fn position(&mut self) -> Result<StoredPosition, Self::Error>;

    /// Fetch the current forgotten version.
    async fn forgotten(&mut self) -> Result<Forgotten, Self::Error>;

    /// Get the full list of all internal hashes stored, indexed by position and height.
    #[allow(clippy::type_complexity)]
    fn hashes(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, u8, Hash), Self::Error>> + Send + '_>>;

    /// Get the full list of all commitments stored, indexed by position.
    #[allow(clippy::type_complexity)]
    fn commitments(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, Commitment), Self::Error>> + Send + '_>>;
}

/// A storage backend capable of writing [`struct@Hash`]es and [`Commitment`]s, and
/// garbage-collecting those which have been forgotten.
#[async_trait]
pub trait Write: Read {
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
