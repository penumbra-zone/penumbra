use crate::{GetHash, Hash, Height};

/// Either an item or just its hash, to be used when inserting into a tree.
///
/// When inserting, only items inserted with [`Insert::Keep`] are retained as witnessed leaves of
/// the tree; those inserted with [`Insert::Hash`] are pruned.
#[derive(Debug, Clone, Copy, Eq)]
pub enum Insert<T> {
    /// An item unto itself: when inserting, keep this witnessed in the tree.
    Keep(T),
    /// The hash of an item: when inserting, don't keep this witnessed in the tree.
    Hash(Hash),
}

impl<T> Insert<T> {
    /// Transform a `&Insert<T>` into a `Insert<&T>`.
    #[inline]
    pub fn as_ref(&self) -> Insert<&T> {
        match self {
            Insert::Keep(item) => Insert::Keep(item),
            Insert::Hash(hash) => Insert::Hash(*hash),
        }
    }

    /// Map a function over the [`Insert::Keep`] part of an `Insert<T>`.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Insert<U> {
        match self {
            Insert::Keep(item) => Insert::Keep(f(item)),
            Insert::Hash(hash) => Insert::Hash(hash),
        }
    }
}

impl<T: PartialEq<S>, S> PartialEq<Insert<S>> for Insert<T> {
    fn eq(&self, other: &Insert<S>) -> bool {
        match (self, other) {
            (Insert::Keep(item), Insert::Keep(other)) => item == other,
            (Insert::Hash(hash), Insert::Hash(other)) => hash == other,
            _ => false,
        }
    }
}

impl<T: GetHash> GetHash for Insert<T> {
    #[inline]
    fn hash(&self) -> Hash {
        match self {
            Insert::Keep(item) => item.hash(),
            Insert::Hash(hash) => *hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match self {
            Insert::Keep(item) => item.cached_hash(),
            Insert::Hash(hash) => Some(*hash),
        }
    }
}

/// An active tree supporting the insertion of new elements and the updating of the
/// most-recently-inserted element.
pub trait Active: Focus + Sized {
    /// The type of item to persist in each witnessed leaf of the active tree.
    type Item: Focus;

    /// Make a new [`Active`] containing a single [`struct@Hash`] or `Self::Item`.
    fn singleton(item: Insert<Self::Item>) -> Self;

    /// Insert a new [`struct@Hash`] or `Self::Item` into this [`Active`], returning either `Self` with the
    /// thing inserted, or the un-inserted thing and the [`Complete`] of this [`Active`].
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

pub mod path;

/// An authentication path into the `Tree`.
///
/// This is statically guaranteed to have the same length as the height of the tree.
pub type AuthPath<Tree> = <<Tree as Height>::Height as path::Path>::Path;

/// Witness an authentication path into a tree, or remove a witnessed item from one.
pub trait Witness: Height + Sized {
    /// Witness an authentication path to the given index in the tree.
    ///
    /// The input mutable slice should be at least the height of the tree, and is overwritten by
    /// this function.
    fn witness(&self, index: usize) -> Option<(AuthPath<Self>, Hash)>;

    /// Remove the witness for the given index.
    ///
    /// Returns either `(Self, boool)` where the boolean is `true` if the witness was removed or
    /// `false` if the witness was not present, or `Hash` if the witness was removed and it was the
    /// last witness remaining in this tree.
    fn remove_witness(self, index: usize) -> Result<(Self, bool), Hash>;
}

/// A proof of inclusion for a single commitment in a tree.
pub struct Proof<Tree: Height> {
    index: usize,
    auth_path: AuthPath<Tree>,
    leaf: Hash,
}

impl<Tree: Height> Proof<Tree> {
    /// Verify a [`Proof`] of inclusion against the root hash of a tree.
    ///
    /// Returns `true` if and only if the proof was valid for this root.
    pub fn verify(&self, root: Hash) -> bool {
        root == <Tree::Height as path::Path>::root(&self.auth_path, self.index, self.leaf)
    }

    /// Create a proof of inclusion for the given index in the tree, or return `None` if the index
    /// does not have a witness in the tree.
    pub fn of_inclusion(index: usize, tree: &Tree) -> Option<Self>
    where
        Tree: Witness,
    {
        tree.witness(index).map(|(auth_path, leaf)| Self {
            index,
            auth_path,
            leaf,
        })
    }
}
