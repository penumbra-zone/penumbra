use std::{fmt::Debug, mem};

use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        frontier::{Forget, Full},
        path::Witness,
    },
    AuthPath, Focus, ForgetOwned, Frontier, GetHash, Hash, Height, Insert,
};

use super::super::{complete, frontier};

/// A frontier of a tier of the tiered commitment tree, being an 8-deep quad-tree of items.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Default(bound = ""),
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
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
pub enum Inner<Item: Focus> {
    /// Either an empty tree (`None`) or a tree with at least one element in it.
    Frontier(Box<Option<Nested<Item>>>),
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

impl<Item: Focus> Default for Inner<Item> {
    fn default() -> Self {
        Inner::Frontier(Box::new(None))
    }
}

impl<Item: Focus> Tier<Item> {
    /// Create a new frontier tier.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an item or its hash into this frontier tier.
    ///
    /// If the tier is full, return the input item without inserting it.
    pub fn insert(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        match &mut self.inner {
            // The tier is full or is a single hash, so return the item without inserting it
            Inner::Complete(_) | Inner::Hash(_) => Err(item),
            // The tier is a frontier, so try inserting into it
            Inner::Frontier(incomplete) => match mem::take(&mut **incomplete) {
                None => {
                    // The tier is empty, so insert the item
                    **incomplete = Some(Nested::singleton(item));
                    Ok(())
                }
                Some(frontier) => match frontier.insert(item) {
                    // The insertion succeeded, so we're still frontier
                    Ok(frontier) => {
                        **incomplete = Some(frontier);
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

    /// Update the currently focused `Insert<Item>` (i.e. the
    /// most-recently-[`insert`](Self::insert)ed one), returning the result of the function.
    ///
    /// If there is no currently focused `Insert<Item>`, the function is called on `None`.
    pub fn update<T>(&mut self, f: impl FnOnce(Option<&mut Insert<Item>>) -> T) -> T {
        if let Inner::Frontier(frontier) = &mut self.inner {
            if let Some(ref mut frontier) = **frontier {
                frontier.update(|item| f(Some(item)))
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
    pub fn focus(&self) -> Option<&Insert<Item>> {
        if let Inner::Frontier(frontier) = &self.inner {
            frontier.as_ref().as_ref().map(|frontier| frontier.focus())
        } else {
            None
        }
    }

    /// Check if this [`Tier`] is empty.
    pub fn is_empty(&self) -> bool {
        if let Inner::Frontier(frontier) = &self.inner {
            frontier.is_none()
        } else {
            false
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
            Inner::Frontier(frontier) => frontier
                .as_ref()
                .as_ref()
                .map(|frontier| frontier.hash())
                .unwrap_or_else(Hash::default),
            Inner::Complete(complete) => complete.hash(),
            Inner::Hash(hash) => *hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match &self.inner {
            Inner::Frontier(frontier) => frontier
                .as_ref()
                .as_ref()
                .map(|frontier| frontier.cached_hash())
                .unwrap_or_else(|| Some(Hash::default())),
            Inner::Complete(complete) => complete.cached_hash(),
            Inner::Hash(hash) => Some(*hash),
        }
    }
}

impl<Item: Focus> Focus for Tier<Item> {
    type Complete = complete::Tier<Item::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        match self.inner {
            Inner::Frontier(frontier) => {
                if let Some(frontier) = *frontier {
                    match frontier.finalize() {
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

impl<Item: Focus + Witness> Witness for Tier<Item>
where
    Item::Complete: Witness<Item = Item::Item>,
{
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        match &self.inner {
            Inner::Frontier(frontier) => frontier
                .as_ref()
                .as_ref()
                .and_then(|frontier| frontier.witness(index)),
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
        // Replace `self.inner` temporarily with an empty hash, so we can move out of it
        let inner = mem::replace(&mut self.inner, Inner::Hash(Hash::default()));

        // Whether something was actually forgotten
        let forgotten;

        // No matter which branch we take, we always put something valid back into `self.inner` before
        // returning from this function
        (forgotten, self.inner) = match inner {
            // If the tier is a frontier, try to forget from the frontier path, if it's not empty
            Inner::Frontier(mut frontier) => (
                if let Some(ref mut frontier) = *frontier {
                    frontier.forget(index)
                } else {
                    false
                },
                Inner::Frontier(frontier),
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
    fn check_inner_size() {
        static_assertions::assert_eq_size!(Tier<Tier<Tier<crate::Item>>>, [u8; 64]);
    }
}
