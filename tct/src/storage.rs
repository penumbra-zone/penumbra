//! Incremental serialization and non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Range,
    pin::Pin,
};

use ark_ed_on_bls12_377::Fq;
use futures::{stream, Stream};

pub mod deserialize;
pub mod in_memory;
pub mod serialize;
pub use in_memory::InMemory;

/// Proptest generators for things relevant to construction.
#[cfg(feature = "arbitrary")]
pub mod arbitrary;

/// A point in the serialized tree: the hash or commitment (represented by an [`Fq`]), and its
/// position and depth in the tree.
///
/// The depth is the distance from the root, so leaf hashes have depth 24, and commitments
/// themselves have depth 25.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// The position of the value.
    pub position: u64,
    /// The depth of the value from the root of the tree.
    ///
    /// Note that this representation means that leaf hashes have depth 24, and commitments
    /// themselves have depth 25.
    pub depth: u8,
    /// The value at this point.
    pub here: Fq,
}

impl Point {
    /// Get the range of positions "beneath" this point.
    pub fn range(&self) -> Range<u64> {
        let height = 24u8.saturating_sub(self.depth);
        let stride = 4u64.pow(height.into());
        self.position..(self.position + stride).min(4u64.pow(24) - 1)
    }
}

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
    async fn position(&mut self) -> Result<u64, Self::Error>;

    /// Read a particular point in the storage, or return `None` if it is not represented.
    ///
    /// This is not used for batch deserialization; it's used only for testing and error checking.
    async fn read(&mut self, position: u64, depth: u8) -> Result<Option<Fq>, Self::Error>;

    /// Get the full list of all [`Point`]s stored, ordered lexicographically by **position** and
    /// then by **depth**.
    fn points(&mut self) -> Pin<Box<dyn Stream<Item = Result<Point, Self::Error>> + '_>>;
}

/// A storage backend capable of writing [`Point`]s, and garbage-collecting those which have been
/// forgotten.
#[async_trait]
pub trait Write: Read {
    /// Write a single point into storage.
    ///
    /// This should return an error if the point is already present; no point's value should ever
    /// be overwritten.
    async fn write(&mut self, point: Point) -> Result<(), Self::Error>;

    /// Delete every stored [`Point`] whose *depth* is greater than `minimum_depth` and whose
    /// **position** is within the half-open [`Range`] of `positions`.
    async fn delete_range(
        &mut self,
        minimum_depth: u8,
        positions: Range<u64>,
    ) -> Result<(), Self::Error>;

    /// Set the stored position of the tree.
    async fn set_position(&mut self, position: u64) -> Result<(), Self::Error>;
}
