use crate::{AuthPath, GetHash, Height, Insert};

/// An active tree supporting the insertion of new elements and the updating of the
/// most-recently-inserted element.
pub trait Active: Focus + Sized {
    /// The type of item to persist in each witnessed leaf of the active tree.
    type Item: Focus;

    /// Make a new [`Active`] containing a single [`Hash`](crate::Hash) or `Self::Item`.
    fn singleton(item: Insert<Self::Item>) -> Self;

    /// Insert a new [`Hash`](crate::Hash) or `Self::Item` into this [`Active`], returning either
    /// `Self` with the thing inserted, or the un-inserted thing and the [`Complete`] of this
    /// [`Active`].
    fn insert(self, item: Insert<Self::Item>) -> Result<Self, Full<Self>>;

    /// Update the currently active `Insert<Self::Item>` (i.e. the most-recently
    /// [`insert`](Active::insert)ed one), returning the result of the function.
    fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Self::Item>) -> T) -> T;

    /// Get a reference to the focused `Insert<Self::Item>` (i.e. the most-recently
    /// [`insert`](Active::insert)ed one).
    fn focus(&self) -> &Insert<Self::Item>;
}

/// A type which can be the focus of an [`Active`] tree: it can be finalized to make a [`Complete`]
/// tree.
pub trait Focus: Height<Height = <Self::Complete as Height>::Height> + GetHash {
    /// The [`Complete`] of this [`Active`].
    type Complete: Complete<Focus = Self>;

    /// Transition from an [`Active`] to being [`Complete`].
    fn finalize(self) -> Insert<Self::Complete>;
}

/// Marker trait for a type which is the frozen completion of some [`Focus`]ed insertion point.
///
/// It is enforced by the type system that [`Complete`] and [`Focus`] are dual to one another.
pub trait Complete: Height + GetHash {
    /// The [`Focus`] of this [`Complete`].
    type Focus: Focus<Complete = Self>;
}

/// The result of [`Active::insert`] when the [`Active`] is full.
pub struct Full<T: Active> {
    /// The original hash or item that could not be inserted.
    pub item: Insert<T::Item>,
    /// The completed structure, which has no more room for any further insertions, or a hash of
    /// that structure if it contained no witnesses.
    pub complete: Insert<T::Complete>,
}
/// Witness an authentication path into a tree, or remove a witnessed item from one.
pub trait Witness: Height + Sized {
    /// The leaf of the tree: the element being witnessed.
    type Item;

    /// Witness an authentication path to the given index in the tree.
    ///
    /// The input mutable slice should be at least the height of the tree, and is overwritten by
    /// this function.
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)>;
}

/// Forget about the authentication path to a given index.
pub trait Forget: Height {
    /// Remove the witness for the given index.
    ///
    /// Returns `true` if the witness was previously present in the tree.
    fn forget(&mut self, index: impl Into<u64>) -> bool;
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
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool);
}
