use crate::prelude::*;

/// A complete, witnessed leaf of a tree.
#[derive(Clone, Copy, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item>(pub(in super::super) Item);

impl<Item> Leaf<Item> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: Item) -> Self {
        Self(item)
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
    type Focus = frontier::Leaf<<Item as Complete>::Focus>;
}

impl<Item: Witness> Witness for Leaf<Item> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        self.0.witness(index)
    }

    #[inline]
    fn foreach_witness(&self, per_witness: impl FnMut(u64, Hash)) {
        self.0.foreach_witness(per_witness)
    }
}

impl<Item: ForgetOwned> ForgetOwned for Leaf<Item> {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        let (item, forgotten) = self.0.forget_owned(index);
        (item.map(Leaf), forgotten)
    }
}

impl<Item: Height + GetHash + Traverse> Visit for Leaf<Item> {
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output {
        visitor.complete_leaf(index, self)
    }
}

impl<Item: Height + GetHash + Traverse> Traverse for Leaf<Item> {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        traversal.traverse_complete(visitor, output, self, [&self.0]);
    }
}
