use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        hash,
        path::{self, Witness},
    },
    AuthPath, Complete, ForgetOwned, GetHash, Hash, Height,
};

use super::super::active;

type N<Child, Hasher> = super::super::complete::Node<Child, Hasher>;
type L<Item> = super::super::complete::Leaf<Item>;

/// An eight-deep complete tree with the given item at each leaf.
pub type Nested<Item, H> = N<N<N<N<N<N<N<N<L<Item>, H>, H>, H>, H>, H>, H>, H>, H>;
// Count the levels:       1 2 3 4 5 6 7 8

/// A complete tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(
    PartialEq(bound = "Item: Height + PartialEq"),
    Eq(bound = "Item: Height + Eq")
)]
pub struct Tier<Item, Hasher> {
    pub(in super::super) inner: Nested<Item, Hasher>,
}

impl<Item: Height, Hasher> Height for Tier<Item, Hasher> {
    type Height = <Nested<Item, Hasher> as Height>::Height;
}

impl<Item: Height + GetHash<Hasher>, Hasher: hash::Hasher> GetHash<Hasher> for Tier<Item, Hasher> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        self.inner.cached_hash()
    }
}

impl<Item: Complete<Hasher>, Hasher: hash::Hasher> Complete<Hasher> for Tier<Item, Hasher> {
    type Focus = active::Tier<Item::Focus, Hasher>;
}

impl<Item: Witness<Hasher> + GetHash<Hasher>, Hasher: hash::Hasher> Witness<Hasher>
    for Tier<Item, Hasher>
where
    Item::Height: path::Path<Hasher>,
{
    type Item = Item::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Self::Item)> {
        self.inner.witness(index)
    }
}

impl<Item: ForgetOwned<Hasher>, Hasher: hash::Hasher> ForgetOwned<Hasher> for Tier<Item, Hasher> {
    fn forget_owned(self, index: impl Into<u64>) -> (crate::Insert<Self, Hasher>, bool) {
        let (inner, forgotten) = self.inner.forget_owned(index);
        (inner.map(|inner| Tier { inner }), forgotten)
    }
}
