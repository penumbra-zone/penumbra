#[macro_use]
extern crate derivative;

mod three;
use three::{Elems, Three};

pub mod hash;
use hash::{GetHash, Hash};

pub mod leaf;
pub mod level;
pub mod node;

/// Trait identifying the statically-known height of a given tree element.
///
/// This is used to differentiate the hashes at each level of the tree.
trait Height {
    /// The height of this type above the leaves of the tree.
    const HEIGHT: usize;
}

trait Active: Height + GetHash + Sized {
    type Item;
    type Complete: Complete<Active = Self>;

    /// Make a new [`Active`] containing a single [`Hash`] or `Self::Item`.
    fn singleton(item: HashOr<Self::Item>) -> Self;

    /// Insert a new [`Hash`] or `Self::Item` into this [`Active`], returning either `Self` with the
    /// thing inserted, or the un-inserted thing and the [`Complete`] of this [`Active`].
    fn insert(self, item: HashOr<Self::Item>) -> Result<Self, Full<Self::Item, Self::Complete>>;

    /// Alter the currently active `Self::Item` (i.e. the most-recently [`insert`](Active::insert)ed
    /// one), returning the result of the function. This does nothing if the most-recently inserted
    /// thing was a [`Hash`].
    ///
    /// # Correctness
    ///
    /// If the function is invoked on a `Self::Item`, this function *must* return `Some(T)`. It is a
    /// violation of this condition to return `None` if the function was called. This condition is
    /// required because internally cached hashes are only cleared if the function was actually
    /// called, and the return value of `None` should be used only to indicate that these caches do
    /// not need to be updated.
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T>;

    /// Transform this [`Active`] into a [`Complete`].
    fn complete(self) -> HashOr<Self::Complete>;
}

/// Marker trait identifying a type which is the frozen completion of some [`Active`] insertion
/// point.
///
/// It is enforced by the type system that [`Complete`] and [`Active`] are dual to one another.
trait Complete: Height + GetHash {
    type Active: Active<Complete = Self>;
}

/// The result of [`Active::insert`] when the [`Active`] is full.
pub struct Full<Item, Complete> {
    /// The original hash or item that could not be inserted.
    pub item: HashOr<Item>,
    /// The completed structure, which has no more room for any further insertions.
    pub complete: HashOr<Complete>,
}

/// Either a hash or some item, or the item itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashOr<T> {
    /// The hash of an item.
    Hash(Hash),
    /// The item itself.
    Item(T),
}

pub struct Commitment;

impl GetHash for Commitment {
    fn hash(&self) -> Hash {
        Hash
    }
}
