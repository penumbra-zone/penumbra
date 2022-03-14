use crate::{Complete, GetHash, Hash, Height, Witness};

use super::super::active;

/// A complete, witnessed leaf of a tree.
#[derive(Clone, Copy, PartialEq, Eq, Derivative)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item>(pub(in super::super) Item);

impl<Item> Leaf<Item> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

impl<Item: Complete> PartialEq<active::Leaf<Item::Focus>> for Leaf<Item>
where
    Item::Focus: PartialEq<Item>,
{
    fn eq(&self, other: &active::Leaf<Item::Focus>) -> bool {
        other == self
    }
}

impl<Item: GetHash> GetHash for Leaf<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.0.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.0.cached_hash()
    }
}

impl<Item: Height> Height for Leaf<Item> {
    type Height = Item::Height;
}

impl<Item: Complete> Complete for Leaf<Item> {
    type Focus = active::Leaf<<Item as crate::Complete>::Focus>;
}

impl<Item: Witness> Witness for Leaf<Item> {
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(crate::AuthPath<Self>, Self::Item)> {
        self.0.witness(index)
    }
}
