use std::{fmt::Debug, mem};

use crate::{Active, Focus, Full, GetHash, Hash, Height, Insert};

use super::super::{active, complete};

/// An active tier of the tiered commitment tree, being an 8-deep quad-tree of items.
#[derive(Derivative)]
#[derivative(Debug(bound = "Item: Debug, Item::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, Item::Complete: Clone"))]
#[derivative(PartialEq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
pub struct Tier<Item: Focus> {
    len: u16,
    witnessed: u16,
    inner: Box<Inner<Item>>,
}

type N<Focus> = active::Node<Focus>;
type L<Item> = active::Leaf<Item>;

/// An eight-deep active tree with the given item stored in each leaf.
pub type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// Count the levels:    1 2 3 4 5 6 7 8

/// The inside of an active level.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
enum Inner<Item: Focus> {
    /// The starting state: an empty tree.
    Empty,
    /// A tree with at least one element in it, which could potentially allow more to be inserted.
    Active(Nested<Item>),
    /// A tree which has been filled, so no more elements can be inserted.
    ///
    /// This is one of two final states; the other is [`Inner::Hash`].
    Complete(complete::tier::Nested<Item::Complete>),
    /// A tree which has been filled, but which witnessed no elements, so it is represented by a
    /// single hash.
    ///
    /// This is one of two final states: the other is [`Inner::Complete`].
    Hash(Hash),
}

impl<Item: Focus> PartialEq for Inner<Item>
where
    Item: PartialEq + PartialEq<Item::Complete>,
    Item::Complete: PartialEq,
{
    fn eq(&self, other: &Inner<Item>) -> bool {
        match (self, other) {
            // Empty tiers are always equal to each other
            (Inner::Empty, Inner::Empty) => true,
            // An empty tier is never equal to a non-empty tier
            (Inner::Empty, Inner::Active(_))
            | (Inner::Empty, Inner::Complete(_))
            | (Inner::Empty, Inner::Hash(_))
            | (Inner::Active(_), Inner::Empty)
            | (Inner::Complete(_), Inner::Empty)
            | (Inner::Hash(_), Inner::Empty)
            // A non-empty, non-hash tier is never equal to a hash tier (because one has witnesses
            // and the other does not)
            | (Inner::Active(_), Inner::Hash(_))
            | (Inner::Complete(_), Inner::Hash(_))
            | (Inner::Hash(_), Inner::Active(_))
            | (Inner::Hash(_), Inner::Complete(_)) => false,
            // Two non-empty, non-hash tiers are equal if their trees are equal (this relies on the
            // `==` implementation between the two inner trees, which is heterogeneous in the case
            // between `Active` and `Complete`)
            (Inner::Active(l), Inner::Active(r)) => l == r,
            (Inner::Active(l), Inner::Complete(r)) => l == r,
            (Inner::Complete(l), Inner::Active(r)) => l == r,
            (Inner::Complete(l), Inner::Complete(r)) => l == r,
            // Two tiers with no witnesses are equal if their hashes are equal
            (Inner::Hash(l), Inner::Hash(r)) => l == r,
        }
    }
}

#[cfg(test)]
#[test]
fn check_eq_impl() {
    static_assertions::assert_impl_all!(Tier<Tier<Tier<crate::Item>>>: Eq);
    static_assertions::assert_impl_all!(Tier<Tier<Tier<Hash>>>: Eq);
}

impl<Item: Focus> PartialEq<complete::Tier<Item::Complete>> for Tier<Item>
where
    Item: PartialEq + PartialEq<Item::Complete>,
    Item::Complete: PartialEq,
{
    fn eq(&self, complete::Tier { inner: r }: &complete::Tier<Item::Complete>) -> bool {
        match *self.inner {
            // Complete tiers are never empty, an empty or hash-only tier is never equal to one,
            // because they don't have witnesses
            Inner::Empty | Inner::Hash(_) => false,
            // Active tiers are equal to complete tiers if their trees are equal (relying on
            // heterogeneous equality between `Active` and `Complete`)
            Inner::Active(ref l) => l == r,
            Inner::Complete(ref l) => l == r,
        }
    }
}

impl<Item: Focus> Default for Inner<Item> {
    fn default() -> Self {
        Inner::Empty
    }
}

impl<Item: Focus> Tier<Item> {
    /// Create a new active tier.
    pub fn new() -> Self {
        Self {
            len: 0,
            witnessed: 0,
            inner: Box::new(Inner::default()),
        }
    }

    /// Insert an item or its hash into this active tier.
    ///
    /// If the tier is full, return the input item without inserting it.
    pub fn insert(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        match mem::take(&mut *self.inner) {
            Inner::Empty => {
                *self.inner = Inner::Active(Nested::singleton(item));
                self.len += 1;
                Ok(())
            }
            Inner::Active(active) => match active.insert(item) {
                Ok(active) => {
                    *self.inner = Inner::Active(active);
                    self.len += 1;
                    Ok(())
                }
                Err(Full { item, complete }) => {
                    *self.inner = match complete {
                        Insert::Hash(hash) => Inner::Hash(hash),
                        Insert::Keep(complete) => Inner::Complete(complete),
                    };
                    Err(item)
                }
            },
            Inner::Complete(complete) => {
                *self.inner = Inner::Complete(complete);
                Err(item)
            }
            Inner::Hash(hash) => {
                *self.inner = Inner::Hash(hash);
                Err(item)
            }
        }
    }

    /// Update the currently active `Insert<Item>` (i.e. the
    /// most-recently-[`insert`](Self::insert)ed one), returning the result of the function.
    ///
    /// If there is no currently active `Insert<Item>` (in the case that the tier is empty or full),
    /// the function is not called, and `None` is returned.
    pub fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Item>) -> T) -> Option<T> {
        if let Inner::Active(active) = &mut *self.inner {
            Some(active.update(f))
        } else {
            None
        }
    }

    /// Get a reference to the focused `Insert<Item>`, if there is one.
    ///
    /// If there is no focused `Insert<Item>` (in the case that the tier is empty or full), `None`
    /// is returned.
    pub fn focus(&self) -> Option<&Insert<Item>> {
        if let Inner::Active(active) = &*self.inner {
            Some(active.focus())
        } else {
            None
        }
    }

    /// Get the total number of insertions performed on this [`Tier`].
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Get the number of items stored in this [`Tier`].
    ///
    /// This will be less than [`Tier::len`] if some hashes were inserted via [`Insert::Hash`].
    pub fn size(&self) -> u16 {
        self.witnessed
    }

    /// Check if this [`Tier`] is empty.
    pub fn is_empty(&self) -> bool {
        matches!(*self.inner, Inner::Empty)
    }
}

impl<Item: Focus> Default for Tier<Item> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Item: Focus> Height for Tier<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Focus> GetHash for Tier<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        match &*self.inner {
            Inner::Empty => Hash::default(),
            Inner::Active(active) => active.hash(),
            Inner::Complete(complete) => complete.hash(),
            Inner::Hash(hash) => *hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match &*self.inner {
            Inner::Empty => Some(Hash::default()),
            Inner::Active(active) => active.cached_hash(),
            Inner::Complete(complete) => complete.cached_hash(),
            Inner::Hash(hash) => Some(*hash),
        }
    }
}

impl<Item: Focus> Focus for Tier<Item> {
    type Complete = complete::Tier<Item::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        match *self.inner {
            Inner::Empty => Insert::Hash(Hash::default()),
            Inner::Active(active) => match active.finalize() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(inner) => Insert::Keep(complete::Tier { inner }),
            },
            Inner::Complete(inner) => Insert::Keep(complete::Tier { inner }),
            Inner::Hash(hash) => Insert::Hash(hash),
        }
    }
}
