use crate::{
    internal::{
        active::{Forget, Full},
        path::Witness,
    },
    Active, AuthPath, Focus, GetHash, Hash, Height, Insert,
};

use super::super::complete;

/// The active (rightmost) leaf in an active tree.
///
/// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to contain
/// the inserted item.
#[derive(Clone, Copy, PartialEq, Eq, Derivative)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item> {
    item: Insert<Item>,
}

impl<Item: Focus> PartialEq<complete::Leaf<Item::Complete>> for Leaf<Item>
where
    Item: PartialEq<Item::Complete>,
{
    fn eq(&self, complete::Leaf(other): &complete::Leaf<Item::Complete>) -> bool {
        match self.item {
            Insert::Keep(ref item) => item == other,
            Insert::Hash(_) => false,
        }
    }
}

impl<Item: GetHash> GetHash for Leaf<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        match self.item {
            Insert::Hash(hash) => hash,
            Insert::Keep(ref item) => item.hash(),
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match self.item {
            Insert::Hash(hash) => Some(hash),
            Insert::Keep(ref item) => item.cached_hash(),
        }
    }
}

impl<Item: Height> Height for Leaf<Item> {
    type Height = Item::Height;
}

impl<Item: Focus> Active for Leaf<Item> {
    type Item = Item;

    #[inline]
    fn singleton(item: Insert<Self::Item>) -> Self {
        Self { item }
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Self::Item>) -> T) -> T {
        f(&mut self.item)
    }

    #[inline]
    fn focus(&self) -> &Insert<Self::Item> {
        &self.item
    }

    #[inline]
    /// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to
    /// contain the inserted item.
    fn insert(self, item: Insert<Self::Item>) -> Result<Self, Full<Self>> {
        Err(Full {
            item,
            complete: self.finalize(),
        })
    }
}

impl<Item: Focus> Focus for Leaf<Item> {
    type Complete = complete::Leaf<<Item as Focus>::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        match self.item {
            Insert::Hash(hash) => Insert::Hash(hash),
            Insert::Keep(item) => match item.finalize() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(item) => Insert::Keep(complete::Leaf::new(item)),
            },
        }
    }
}

impl<Item: Witness> Witness for Leaf<Item> {
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        self.item.as_ref().keep()?.witness(index)
    }
}

impl<Item: Forget> Forget for Leaf<Item> {
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        match self.item {
            Insert::Keep(ref mut item) => item.forget(index),
            Insert::Hash(_) => false,
        }
    }
}
