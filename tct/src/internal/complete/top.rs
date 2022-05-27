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

impl<Item: Height + GetHash> Visit for Top<Item> {
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output {
        visitor.complete_top(index, self)
    }
}

impl<Item: Height + GetHash + Traverse> Traverse for Top<Item> {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        traversal.traverse_complete(visitor, output, self, [&self.inner]);
    }
}
