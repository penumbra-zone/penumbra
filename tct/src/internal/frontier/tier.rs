use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    internal::{frontier::Forget, path::Witness},
    AuthPath, Focus, ForgetOwned, Frontier, GetHash, Hash, Height, Insert,
};

use super::super::{complete, frontier};

/// A frontier of a tier of the tiered commitment tree, being an 8-deep quad-tree of items.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Debug(bound = "Item: Debug, Item::Complete: Debug"),
    Clone(bound = "Item: Clone, Item::Complete: Clone")
)]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
pub struct Tier<Item: Focus> {
    inner: Inner<Item>,
}

type N<Focus> = frontier::Node<Focus>;
type L<Item> = frontier::Leaf<Item>;

/// An eight-deep frontier tree with the given item stored in each leaf.
pub type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// Count the levels:    1 2 3 4 5 6 7 8

/// The inside of a frontier of a tier.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Debug(bound = "Item: Debug, Item::Complete: Debug"),
    Clone(bound = "Item: Clone, Item::Complete: Clone")
)]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
pub enum Inner<Item: Focus> {
    /// A tree with at least one leaf.
    Frontier(Box<Nested<Item>>),
    /// A completed tree which has at least one witnessed child.
    Complete(complete::Nested<Item::Complete>),
    /// A tree which has been filled, but which witnessed no elements, so it is represented by a
    /// single hash.
    Hash(Hash),
}

impl<Item: Focus> Tier<Item> {
    /// Create a new frontier tier.
    pub fn singleton(item: Insert<Item>) -> Self {
        Self {
            inner: Inner::Frontier(Box::new(Nested::singleton(item))),
        }
    }

    /// Insert an item or its hash into this frontier tier.
    ///
    /// If the tier is full, return the input item without inserting it.
    pub fn insert(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        // Temporarily replace the inside with the zero hash (it will get put back right away, this
        // is just to satisfy the borrow checker)
        let inner = std::mem::replace(&mut self.inner, Inner::Hash(Hash::zero()));

        let result;
        (result, self.inner) = match inner {
            // The tier is full or is a single hash, so return the item without inserting it
            inner @ (Inner::Complete(_) | Inner::Hash(_)) => (Err(item), inner),
            // The tier is a frontier, so try inserting into it
            Inner::Frontier(frontier) => {
                if frontier.is_full() {
                    // Don't even try inserting when we know it will fail: this means that there is *no
                    // implicit finalization* of the frontier, even when it is full
                    (Err(item), Inner::Frontier(frontier))
                } else {
                    // If it's not full, then insert the item into it (which we know will succeed)
                    let inner =
                        Inner::Frontier(Box::new(frontier.insert(item).unwrap_or_else(|_| {
                            panic!("frontier is not full, so insert must succeed")
                        })));
                    (Ok(()), inner)
                }
            }
        };

        result
    }

    /// Update the currently focused `Insert<Item>` (i.e. the
    /// most-recently-[`insert`](Self::insert)ed one), returning the result of the function.
    ///
    /// If there is no currently focused `Insert<Item>`, returns `None`.
    pub fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Item>) -> T) -> Option<T> {
        if let Inner::Frontier(frontier) = &mut self.inner {
            Some(frontier.update(f))
        } else {
            None
        }
    }

    /// Get a reference to the focused `Insert<Item>`, if there is one.
    ///
    /// If there is no focused `Insert<Item>` (in the case that the tier is finalized), `None` is
    /// returned.
    pub fn focus(&self) -> Option<&Insert<Item>> {
        if let Inner::Frontier(frontier) = &self.inner {
            Some(frontier.focus())
        } else {
            None
        }
    }

    /// Finalize this tier so that it is internally marked as complete.
    pub fn finalize(&mut self) {
        // Temporarily replace the inside with the zero hash (it will get put back right away, this
        // is just to satisfy the borrow checker)
        let inner = std::mem::replace(&mut self.inner, Inner::Hash(Hash::zero()));

        self.inner = match inner {
            Inner::Frontier(frontier) => match frontier.finalize_owned() {
                Insert::Hash(hash) => Inner::Hash(hash),
                Insert::Keep(inner) => Inner::Complete(inner),
            },
            inner @ (Inner::Complete(_) | Inner::Hash(_)) => inner,
        };
    }

    /// Check whether this tier has been finalized.
    pub fn is_finalized(&self) -> bool {
        match self.inner {
            Inner::Frontier(_) => false,
            Inner::Complete(_) | Inner::Hash(_) => true,
        }
    }
}

impl<Item: Focus> Height for Tier<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Focus> GetHash for Tier<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        match &self.inner {
            Inner::Frontier(frontier) => frontier.hash(),
            Inner::Complete(complete) => complete.hash(),
            Inner::Hash(hash) => *hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match &self.inner {
            Inner::Frontier(frontier) => frontier.cached_hash(),
            Inner::Complete(complete) => complete.cached_hash(),
            Inner::Hash(hash) => Some(*hash),
        }
    }
}

impl<Item: Focus> Focus for Tier<Item> {
    type Complete = complete::Tier<Item::Complete>;

    #[inline]
    fn finalize_owned(self) -> Insert<Self::Complete> {
        match self.inner {
            Inner::Frontier(frontier) => match frontier.finalize_owned() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(inner) => Insert::Keep(complete::Tier { inner }),
            },
            Inner::Complete(inner) => Insert::Keep(complete::Tier { inner }),
            Inner::Hash(hash) => Insert::Hash(hash),
        }
    }
}

impl<Item: Focus + Witness> Witness for Tier<Item>
where
    Item::Complete: Witness<Item = Item::Item>,
{
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        match &self.inner {
            Inner::Frontier(frontier) => frontier.witness(index),
            Inner::Complete(complete) => complete.witness(index),
            Inner::Hash(_) => None,
        }
    }
}

impl<Item: Focus + Forget> Forget for Tier<Item>
where
    Item::Complete: ForgetOwned,
{
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        // Whether something was actually forgotten
        let forgotten;

        // Temporarily replace the inside with the zero hash (it will get put back right away, this
        // is just to satisfy the borrow checker)
        let inner = std::mem::replace(&mut self.inner, Inner::Hash(Hash::zero()));

        (forgotten, self.inner) = match inner {
            // If the tier is a frontier, try to forget from the frontier path, if it's not empty
            Inner::Frontier(mut frontier) => (frontier.forget(index), Inner::Frontier(frontier)),
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

impl<Item: Focus> From<complete::Tier<Item::Complete>> for Tier<Item> {
    fn from(complete: complete::Tier<Item::Complete>) -> Self {
        Self {
            inner: Inner::Complete(complete.inner),
        }
    }
}

impl<Item: Focus> From<complete::Top<Item::Complete>> for Tier<Item> {
    fn from(complete: complete::Top<Item::Complete>) -> Self {
        Self {
            inner: Inner::Complete(complete.inner),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_inner_size() {
        static_assertions::assert_eq_size!(Tier<Tier<Tier<crate::Item>>>, [u8; 64]);
    }
}
