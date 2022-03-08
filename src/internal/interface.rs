use crate::{GetHash, Hash, Height};

/// Either a hash or some item, or the item itself, for insertion into a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insert<T> {
    /// An item unto itself: when inserting, keep this witnessed in the tree.
    Keep(T),
    /// The hash of an item: when inserting, don't keep this witnessed in the tree.
    Hash(Hash),
}

pub trait Active: Focus + Sized {
    /// The type of item to persist in each witnessed leaf of the active tree.
    type Item;

    /// Make a new [`Active`] containing a single [`Hash`] or `Self::Item`.
    fn singleton(item: Insert<Self::Item>) -> Self;

    /// Insert a new [`Hash`] or `Self::Item` into this [`Active`], returning either `Self` with the
    /// thing inserted, or the un-inserted thing and the [`Complete`] of this [`Active`].
    fn insert(self, item: Insert<Self::Item>) -> Result<Self, Full<Self>>;

    // TODO: can this be replaced with a more ergonomic guard-pattern thing?
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
}

/// Describes a type which can be the focus of an [`Active`] tree: it can be finalized to make a
/// [`Complete`] tree.
pub trait Focus: Height<Height = <Self::Complete as Height>::Height> + GetHash {
    /// The [`Complete`] of this [`Active`].
    type Complete: Complete<Active = Self>;

    /// Transition from an [`Active`] to being [`Complete`].
    fn finalize(self) -> Insert<Self::Complete>;
}

/// A type which is the frozen completion of some [`Focus`]ed insertion point.
///
/// It is enforced by the type system that [`Complete`] and [`Focus`] are dual to one another.
pub trait Complete: Height + GetHash {
    type Active: Focus<Complete = Self>;
}

/// The result of [`Active::insert`] when the [`Active`] is full.
pub struct Full<T: Active> {
    /// The original hash or item that could not be inserted.
    pub item: Insert<T::Item>,
    /// The completed structure, which has no more room for any further insertions, or a hash of
    /// that structure if it contained no witnesses.
    pub complete: Insert<T::Complete>,
}
