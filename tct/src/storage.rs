//! Incremental serialization and non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Range,
    pin::Pin,
};

use ark_ed_on_bls12_377::Fq;
use futures::{stream, Stream};

use crate::prelude::*;

pub mod deserialize;
pub mod in_memory;
pub mod serialize;
pub use in_memory::InMemory;

/// Proptest generators for things relevant to construction.
#[cfg(feature = "arbitrary")]
pub mod arbitrary;

/// In a depth-first traversal, is the next node below, or to the right? If this is the last
/// represented sibling, then we should go up instead of (illegally) right.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// This node is an internal node, with a non-zero number of children and an optional cached
    /// value. We should create it, then continue the traversal to create its children.
    Node {
        /// The element at this leaf, if any (internal nodes can always calculate their element, so
        /// this is optional).
        here: Option<Fq>,
        /// The number of children of this node.
        size: Size,
    },
    /// This node is a leaf, with no children and a mandatory value. We should create it, then
    /// return it as completed, to continue the traversal at the parent.
    Leaf {
        /// The element at this leaf.
        here: Fq,
    },
}

/// The number of children of a node we're creating.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Size {
    /// 1 child.
    One,
    /// 2 children.
    Two,
    /// 3 children.
    Three,
    /// 4 children.
    Four,
}

impl From<Size> for usize {
    fn from(size: Size) -> Self {
        match size {
            Size::One => 1,
            Size::Two => 2,
            Size::Three => 3,
            Size::Four => 4,
        }
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        usize::from(*self).fmt(f)
    }
}

/// A storage backend capable of reading [`Point`]s, as well as storing the current position.
#[async_trait]
pub trait Read {
    /// The error returned when something goes wrong in a request.
    type Error;

    /// Fetch the current position stored.
    async fn position(&mut self) -> Result<Option<Position>, Self::Error>;

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
    ///
    /// This should return an error if a hash is already present at that location; no location's
    /// value should ever be overwritten.
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

    /// Delete every stored [`Point`] whose height is greater than `below_height` and whose
    /// position is within the half-open [`Range`] of `positions`.
    async fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error>;

    /// Set the stored position of the tree.
    async fn set_position(&mut self, position: Option<Position>) -> Result<(), Self::Error>;
}
