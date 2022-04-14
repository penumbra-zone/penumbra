use serde::{Deserialize, Serialize};

use crate::{
    internal::path::{self, Witness},
    Complete, ForgetOwned, GetHash, Hash, Height, Insert,
};

use super::super::active;

/// A complete, witnessed leaf of a tree.
#[derive(Clone, Copy, PartialEq, Eq, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item>(pub(in super::super) Item);

impl<Item> Leaf<Item> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

impl<Item: Complete<Hasher>, Hasher> PartialEq<active::Leaf<Item::Focus, Hasher>> for Leaf<Item>
where
    Item::Focus: PartialEq<Item>,
{
    fn eq(&self, other: &active::Leaf<Item::Focus, Hasher>) -> bool {
        other == self
    }
}

impl<Item: GetHash<Hasher>, Hasher> GetHash<Hasher> for Leaf<Item> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        self.0.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        self.0.cached_hash()
    }
}

impl<Item: Height> Height for Leaf<Item> {
    type Height = Item::Height;
}

impl<Item: Complete<Hasher>, Hasher> Complete<Hasher> for Leaf<Item> {
    type Focus = active::Leaf<<Item as crate::Complete<Hasher>>::Focus, Hasher>;
}

impl<Item: Witness<Hasher>, Hasher> Witness<Hasher> for Leaf<Item>
where
    Item::Height: path::Path<Hasher>,
{
    type Item = Item::Item;

    fn witness(
        &self,
        index: impl Into<u64>,
    ) -> Option<(crate::AuthPath<Self, Hasher>, Self::Item)> {
        self.0.witness(index)
    }
}

impl<Item: ForgetOwned<Hasher> + GetHash<Hasher>, Hasher> ForgetOwned<Hasher> for Leaf<Item> {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self, Hasher>, bool) {
        let (item, forgotten) = self.0.forget_owned(index);
        (item.map(Leaf), forgotten)
    }
}
