use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        frontier::{Forget, Full},
        height::IsHeight,
        path::Witness,
    },
    AuthPath, Focus, Frontier, GetHash, Hash, Height, Insert,
};

use super::super::complete;

/// The frontier (rightmost) leaf in a frontier of a tree.
///
/// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to contain
/// the inserted item.
#[derive(Clone, Copy, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item> {
    item: Insert<Item>,
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

impl<Item: Focus> Frontier for Leaf<Item> {
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

impl<Item: GetHash + Forget> Forget for Leaf<Item> {
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        match self.item {
            Insert::Keep(ref mut item) => {
                // An optimization: when we know we're at the leaf, instead of recursively
                // delegating to some built-in way to forget an `Item` (which would require
                // additional bookkeeping), we can just forget the item directly by getting out the
                // stored hash and setting `self.item` to that hash.
                //
                // Of note, the `forget` method on `Item` unconditionally panics, but it is never
                // invoked because we short-circuit here.
                if Self::Height::HEIGHT == 0 {
                    self.item = Insert::Hash(item.hash());
                    true
                } else {
                    item.forget(index)
                }
            }
            Insert::Hash(_) => false,
        }
    }
}
