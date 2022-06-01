//! This module contains trait definitions for the entire interface of the internal tree. All of
//! them are exported from either [`frontier`](crate::internal::frontier) or
//! [`complete`](crate::internal::complete), but they are also exported from here for ease of
//! reading.

use crate::prelude::*;

/// A frontier of a tree supporting the insertion of new elements and the updating of the
/// most-recently-inserted element.
pub trait Frontier: Focus + Sized {
    /// The type of item to persist in each witnessed leaf of the frontier.
    type Item;

    /// Make a new [`Frontier`] containing a single [`Hash`](struct@Hash) or `Self::Item`.
    fn new(item: Self::Item) -> Self;

    /// Insert a new [`Hash`](struct@Hash) or `Self::Item` into this [`Frontier`], returning either
    /// `Self` with the thing inserted, or the un-inserted thing and the [`Complete`] of this
    /// [`Frontier`].
    fn insert_owned(self, item: Self::Item) -> Result<Self, Full<Self>>;

    /// Update the currently focused `Insert<Self::Item>` (i.e. the most-recently
    /// [`insert`](Frontier::insert_owned) one), returning the result of the function.
    fn update<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T>;

    /// Get a reference to the focused `Insert<Self::Item>` (i.e. the most-recently
    /// [`insert`](Frontier::insert_owned) one).
    fn focus(&self) -> Option<&Self::Item>;

    /// Check whether this frontier is full.
    fn is_full(&self) -> bool;
}

/// A type which can be the focus of an [`Frontier`] tree: it can be finalized to make a [`Complete`]
/// tree.
pub trait Focus: Height<Height = <Self::Complete as Height>::Height> + GetHash {
    /// The [`Complete`] of this [`Frontier`].
    type Complete: Complete<Focus = Self>;

    /// Transition from an [`Frontier`] to being [`Complete`].
    fn finalize_owned(self) -> Insert<Self::Complete>;
}

/// Marker trait for a type which is the frozen completion of some [`Focus`]ed insertion point.
///
/// It is enforced by the type system that [`Complete`] and [`Focus`] are dual to one another.
pub trait Complete: Height + GetHash {
    /// The corresponding [`Focus`] of this [`Complete`] (i.e. the type which will become this type
    /// when it is [`finalize_owned`](Focus::finalize_owned)).
    type Focus: Focus<Complete = Self>;
}

/// The result of [`Frontier::insert_owned`] when the [`Frontier`] is full.
#[derive(Debug)]
pub struct Full<T: Frontier> {
    /// The original hash or item that could not be inserted.
    pub item: T::Item,
    /// The completed structure, which has no more room for any further insertions, or a hash of
    /// that structure if it contained no witnesses.
    pub complete: Insert<T::Complete>,
}

/// Witness an authentication path into a tree, or remove a witnessed item from one.
pub trait Witness: Height + Sized {
    /// Witness an authentication path to the given index in the tree.
    ///
    /// The input mutable slice should be at least the height of the tree, and is overwritten by
    /// this function.
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)>;
}

/// Get the position of the next insertion into the tree.
pub trait GetPosition {
    /// The position of the next insertion into the tree.
    ///
    /// Returns `None` if the tree is full.
    fn position(&self) -> Option<u64>;
}

/// Forget about the authentication path to a given index.
pub trait Forget: Height {
    /// Remove the witness for the given index.
    ///
    /// Returns `true` if the witness was previously present in the tree.
    fn forget(&mut self, forgotten: Forgotten, index: impl Into<u64>) -> bool;
}

/// Forget about the authentication path to a given index, when forgetting can turn the entirety of
/// `Self` into a hash.
pub trait ForgetOwned: Height + Sized {
    /// Remove the witness for the given index and summarize the item as a single `Hash` if it now
    /// contains no more witnesses.
    ///
    /// Returns either `(Self, boool)` where the boolean is `true` if the witness was removed or
    /// `false` if the witness was not present, or `Hash` if the witness was removed and it was the
    /// last witness remaining in this tree.
    fn forget_owned(self, forgotten: Forgotten, index: impl Into<u64>) -> (Insert<Self>, bool);
}
