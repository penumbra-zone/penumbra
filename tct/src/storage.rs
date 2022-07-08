//! Incremental serialization and non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Range,
    pin::Pin,
};

use futures::{stream, Stream};

use crate::prelude::*;

pub mod deserialize;
pub mod in_memory;
pub mod serialize;
pub use in_memory::InMemory;

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

/// A storage backend capable of reading [`Point`]s, as well as storing the current position.
#[async_trait]
pub trait Read {
    /// The error returned when something goes wrong in a request.
    type Error;

    /// Fetch the current position stored.
    async fn position(&mut self) -> Result<StoredPosition, Self::Error>;

    /// Read a particular hash in the storage, or return `None` if it is not represented.
    ///
    /// This is not used for batch deserialization; it's used only for testing and error checking.
    async fn get_hash(
        &mut self,
        position: Position,
        height: u8,
    ) -> Result<Option<Hash>, Self::Error>;

    /// Read a particular commitment in the storage, or return `None` if it is not represented.
    ///
    /// This is not used for batch deserialization; it's used only for testing and error checking.
    async fn get_commitment(
        &mut self,
        position: Position,
    ) -> Result<Option<Commitment>, Self::Error>;

    /// Get the full list of all internal hashes stored, indexed by position and height.
    fn hashes(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, u8, Hash), Self::Error>> + '_>>;

    /// Get the full list of all commitments stored, indexed by position.
    fn commitments(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, Commitment), Self::Error>> + '_>>;
}

/// A storage backend capable of writing [`Point`]s, and garbage-collecting those which have been
/// forgotten.
#[async_trait]
pub trait Write: Read {
    /// Write a single hash into storage.
    async fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
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

    /// Delete every stored [`Point`] whose height is less than `below_height` and whose
    /// position is within the half-open [`Range`] of `positions`.
    async fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error>;

    /// Set the stored position of the tree.
    async fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error>;
}
