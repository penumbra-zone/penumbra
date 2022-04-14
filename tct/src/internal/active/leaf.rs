use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        active::{Forget, Full},
        height::IsHeight,
        path::{self, Witness},
    },
    Active, AuthPath, Focus, GetHash, Hash, Height, Insert,
};

use super::super::complete;

/// The active (rightmost) leaf in an active tree.
///
/// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to contain
/// the inserted item.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Debug = "transparent",
    Clone(bound = "Item: Clone"),
    Copy(bound = "Item: Copy"),
    PartialEq(bound = "Item: PartialEq"),
    Eq(bound = "Item: Eq")
)]
pub struct Leaf<Item, Hasher> {
    item: Insert<Item, Hasher>,
}

impl<Item: Focus<Hasher>, Hasher> PartialEq<complete::Leaf<Item::Complete>> for Leaf<Item, Hasher>
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

impl<Item: GetHash<Hasher>, Hasher> GetHash<Hasher> for Leaf<Item, Hasher> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        match self.item {
            Insert::Hash(hash) => hash,
            Insert::Keep(ref item) => item.hash(),
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        match self.item {
            Insert::Hash(hash) => Some(hash),
            Insert::Keep(ref item) => item.cached_hash(),
        }
    }
}

impl<Item: Height, Hasher> Height for Leaf<Item, Hasher> {
    type Height = Item::Height;
}

impl<Item: Focus<Hasher>, Hasher> Active<Hasher> for Leaf<Item, Hasher> {
    type Item = Item;

    #[inline]
    fn singleton(item: Insert<Self::Item, Hasher>) -> Self {
        Self { item }
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Self::Item, Hasher>) -> T) -> T {
        f(&mut self.item)
    }

    #[inline]
    fn focus(&self) -> &Insert<Self::Item, Hasher> {
        &self.item
    }

    #[inline]
    /// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to
    /// contain the inserted item.
    fn insert(self, item: Insert<Self::Item, Hasher>) -> Result<Self, Full<Self, Hasher>> {
        Err(Full {
            item,
            complete: self.finalize(),
        })
    }
}

impl<Item: Focus<Hasher>, Hasher> Focus<Hasher> for Leaf<Item, Hasher> {
    type Complete = complete::Leaf<<Item as Focus<Hasher>>::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete, Hasher> {
        match self.item {
            Insert::Hash(hash) => Insert::Hash(hash),
            Insert::Keep(item) => match item.finalize() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(item) => Insert::Keep(complete::Leaf::new(item)),
            },
        }
    }
}

impl<Item: Witness<Hasher> + GetHash<Hasher>, Hasher> Witness<Hasher> for Leaf<Item, Hasher>
where
    Item::Height: path::Path<Hasher>,
{
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Self::Item)> {
        self.item.as_ref().keep()?.witness(index)
    }
}

impl<Item: GetHash<Hasher> + Forget, Hasher> Forget for Leaf<Item, Hasher> {
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
