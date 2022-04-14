use std::{fmt::Debug, mem};

use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        active::{Forget, Full},
        hash,
        path::{self, Witness},
    },
    Active, AuthPath, Focus, ForgetOwned, GetHash, Hash, Height, Insert,
};

use super::super::{active, complete};

/// An active tier of the tiered commitment tree, being an 8-deep quad-tree of items.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Default(bound = ""))]
#[derivative(Debug(bound = "Item: Debug, Item::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, Item::Complete: Clone"))]
#[derivative(PartialEq(
    bound = "Item: PartialEq + PartialEq<Item::Complete>, Item::Complete: PartialEq"
))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
pub struct Tier<Item: Focus<Hasher>, Hasher: hash::Hasher> {
    inner: Inner<Item, Hasher>,
}

type N<Focus, Hasher> = active::Node<Focus, Hasher>;
type L<Item, Hasher> = active::Leaf<Item, Hasher>;

/// An eight-deep active tree with the given item stored in each leaf.
pub type Nested<Item, H> = N<N<N<N<N<N<N<N<L<Item, H>, H>, H>, H>, H>, H>, H>, H>, H>;
// Count the levels:       1 2 3 4 5 6 7 8

/// The inside of an active level.
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
pub enum Inner<Item: Focus<Hasher>, Hasher: hash::Hasher> {
    /// Either an empty tree (`None`) or a tree with at least one element in it.
    Active(Box<Option<Nested<Item, Hasher>>>),
    /// A tree which has been filled, so no more elements can be inserted.
    ///
    /// This is one of two final states; the other is [`Inner::Hash`].
    Complete(complete::tier::Nested<Item::Complete, Hasher>),
    /// A tree which has been filled, but which witnessed no elements, so it is represented by a
    /// single hash.
    ///
    /// This is one of two final states: the other is [`Inner::Complete`].
    Hash(Hash<Hasher>),
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> PartialEq for Inner<Item, Hasher>
where
    Item: PartialEq + PartialEq<Item::Complete>,
    Item::Complete: PartialEq,
{
    fn eq(&self, other: &Inner<Item, Hasher>) -> bool {
        match (self, other) {
            // A non-empty, non-hash tier is never equal to a hash tier (because one has witnesses
            // and the other does not)
            (Inner::Active(_), Inner::Hash(_))
            | (Inner::Complete(_), Inner::Hash(_))
            | (Inner::Hash(_), Inner::Active(_))
            | (Inner::Hash(_), Inner::Complete(_)) => false,
            // Two non-empty, non-hash tiers are equal if their trees are equal (this relies on the
            // `==` implementation between the two inner trees, which is heterogeneous in the case
            // between `Active` and `Complete`)
            (Inner::Active(l), Inner::Active(r)) => l == r,
            (Inner::Complete(l), Inner::Complete(r)) => l == r,
            (Inner::Active(active), Inner::Complete(complete))
            | (Inner::Complete(complete), Inner::Active(active)) => {
                if let Some(active) = &**active {
                    active == complete
                } else {
                    // Empty tiers are never equal to complete tiers, because complete tiers are
                    // never empty
                    false
                }
            }
            // Two tiers with no witnesses are equal if their hashes are equal
            (Inner::Hash(l), Inner::Hash(r)) => l == r,
        }
    }
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> PartialEq<complete::Tier<Item::Complete, Hasher>>
    for Tier<Item, Hasher>
where
    Item: PartialEq + PartialEq<Item::Complete>,
    Item::Complete: PartialEq,
{
    fn eq(
        &self,
        complete::Tier { inner: complete }: &complete::Tier<Item::Complete, Hasher>,
    ) -> bool {
        match self.inner {
            // Complete tiers are never empty, an empty or hash-only tier is never equal to one,
            // because they don't have witnesses
            Inner::Hash(_) => false,
            // Active tiers are equal to complete tiers if their trees are equal (relying on
            // heterogeneous equality between `Active` and `Complete`)
            Inner::Active(ref active) => {
                if let Some(active) = &**active {
                    active == complete
                } else {
                    // Empty tiers are never equal to complete tiers, because complete tiers are never
                    // empty
                    false
                }
            }
            Inner::Complete(ref rhs) => complete == rhs,
        }
    }
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> Default for Inner<Item, Hasher> {
    fn default() -> Self {
        Inner::Active(Box::new(None))
    }
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> Tier<Item, Hasher> {
    /// Create a new active tier.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an item or its hash into this active tier.
    ///
    /// If the tier is full, return the input item without inserting it.
    pub fn insert(&mut self, item: Insert<Item, Hasher>) -> Result<(), Insert<Item, Hasher>> {
        match &mut self.inner {
            // The tier is full or is a single hash, so return the item without inserting it
            Inner::Complete(_) | Inner::Hash(_) => Err(item),
            // The tier is active, so try inserting into it
            Inner::Active(incomplete) => match mem::take(&mut **incomplete) {
                None => {
                    // The tier is empty, so insert the item
                    **incomplete = Some(Nested::singleton(item));
                    Ok(())
                }
                Some(active) => match active.insert(item) {
                    // The insertion succeeded, so we're still active
                    Ok(active) => {
                        **incomplete = Some(active);
                        Ok(())
                    }
                    // The insertion failed, so we need to become complete
                    Err(Full { complete, item }) => {
                        // The tier is full, so set it as complete and return the item without
                        // inserting it
                        self.inner = match complete {
                            Insert::Keep(complete) => Inner::Complete(complete),
                            Insert::Hash(hash) => Inner::Hash(hash),
                        };
                        Err(item)
                    }
                },
            },
        }
    }

    /// Update the currently active `Insert<Item>` (i.e. the
    /// most-recently-[`insert`](Self::insert)ed one), returning the result of the function.
    ///
    /// If there is no currently active `Insert<Item>`, the function is called on `None`.
    pub fn update<T>(&mut self, f: impl FnOnce(Option<&mut Insert<Item, Hasher>>) -> T) -> T {
        if let Inner::Active(active) = &mut self.inner {
            if let Some(ref mut active) = **active {
                active.update(|item| f(Some(item)))
            } else {
                f(None)
            }
        } else {
            f(None)
        }
    }

    /// Get a reference to the focused `Insert<Item>`, if there is one.
    ///
    /// If there is no focused `Insert<Item>` (in the case that the tier is empty or full), `None`
    /// is returned.
    pub fn focus(&self) -> Option<&Insert<Item, Hasher>> {
        if let Inner::Active(active) = &self.inner {
            active.as_ref().as_ref().map(|active| active.focus())
        } else {
            None
        }
    }

    /// Check if this [`Tier`] is empty.
    pub fn is_empty(&self) -> bool {
        if let Inner::Active(active) = &self.inner {
            active.is_none()
        } else {
            false
        }
    }
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> Height for Tier<Item, Hasher> {
    type Height = <Nested<Item, Hasher> as Height>::Height;
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> GetHash<Hasher> for Tier<Item, Hasher> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        match &self.inner {
            Inner::Active(active) => active
                .as_ref()
                .as_ref()
                .map(|active| active.hash())
                .unwrap_or_else(Hash::default),
            Inner::Complete(complete) => complete.hash(),
            Inner::Hash(hash) => *hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        match &self.inner {
            Inner::Active(active) => active
                .as_ref()
                .as_ref()
                .map(|active| active.cached_hash())
                .unwrap_or_else(|| Some(Hash::default())),
            Inner::Complete(complete) => complete.cached_hash(),
            Inner::Hash(hash) => Some(*hash),
        }
    }
}

impl<Item: Focus<Hasher>, Hasher: hash::Hasher> Focus<Hasher> for Tier<Item, Hasher> {
    type Complete = complete::Tier<Item::Complete, Hasher>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete, Hasher> {
        match self.inner {
            Inner::Active(active) => {
                if let Some(active) = *active {
                    match active.finalize() {
                        Insert::Hash(hash) => Insert::Hash(hash),
                        Insert::Keep(inner) => Insert::Keep(complete::Tier { inner }),
                    }
                } else {
                    Insert::Hash(Hash::default())
                }
            }
            Inner::Complete(inner) => Insert::Keep(complete::Tier { inner }),
            Inner::Hash(hash) => Insert::Hash(hash),
        }
    }
}

impl<Item: Focus<Hasher> + Witness<Hasher>, Hasher: hash::Hasher> Witness<Hasher>
    for Tier<Item, Hasher>
where
    Item::Complete: Witness<Hasher, Item = Item::Item>,
    Item::Height: path::Path<Hasher>,
{
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Self::Item)> {
        match &self.inner {
            Inner::Active(active) => active
                .as_ref()
                .as_ref()
                .and_then(|active| active.witness(index)),
            Inner::Complete(complete) => complete.witness(index),
            Inner::Hash(_) => None,
        }
    }
}

impl<Item: Focus<Hasher> + Forget, Hasher: hash::Hasher> Forget for Tier<Item, Hasher>
where
    Item::Complete: ForgetOwned<Hasher>,
{
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        // Replace `self.inner` temporarily with an empty hash, so we can move out of it
        let inner = mem::replace(&mut self.inner, Inner::Hash(Hash::default()));

        // Whether something was actually forgotten
        let forgotten;

        // No matter which branch we take, we always put something valid back into `self.inner` before
        // returning from this function
        (forgotten, self.inner) = match inner {
            // If the tier is active, try to forget from the active path, if it's not empty
            Inner::Active(mut active) => (
                if let Some(ref mut active) = *active {
                    active.forget(index)
                } else {
                    false
                },
                Inner::Active(active),
            ),
            // If the tier is complete, forget from the complete tier and if it resulted in a hash,
            // set the self to that hash
            Inner::Complete(complete) => match complete.forget_owned(index) {
                (Insert::Keep(complete), forgotten) => (forgotten, Inner::Complete(complete)),
                (Insert::Hash(hash), forgotten) => (forgotten, Inner::Hash(hash)),
            },
            // If the tier was just a hash, nothing to do
            Inner::Hash(hash) => (false, Inner::Hash(hash)),
        };

        // Return whether something was actually forgotten
        forgotten
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eq_impl() {
        static_assertions::assert_impl_all!(Tier<Tier<Tier<crate::Item>>>: Eq);
    }

    #[test]
    fn check_inner_size() {
        static_assertions::assert_eq_size!(Tier<Tier<Tier<crate::Item>>>, [u8; 56]);
    }
}
