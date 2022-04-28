use crate::prelude::*;

use complete::Nested;

/// A complete top-level tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Top<Item> {
    pub(in super::super) inner: Nested<Item>,
}

impl<Item: Height> Height for Top<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Height + GetHash> GetHash for Top<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item> From<complete::Tier<Item>> for Top<Item> {
    fn from(tier: complete::Tier<Item>) -> Self {
        Top { inner: tier.inner }
    }
}
