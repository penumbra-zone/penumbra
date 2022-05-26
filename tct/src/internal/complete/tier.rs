use crate::prelude::*;

type N<Child> = super::super::complete::Node<Child>;
type L<Item> = super::super::complete::Leaf<Item>;

/// An eight-deep complete tree with the given item at each leaf.
pub type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// Count the levels:    1 2 3 4 5 6 7 8

/// A complete tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tier<Item> {
    pub(in super::super) inner: Nested<Item>,
}

impl<Item: Height> Height for Tier<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Height + GetHash> GetHash for Tier<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: Complete> Complete for Tier<Item> {
    type Focus = frontier::Tier<Item::Focus>;
}

impl<Item: GetHash + Witness> Witness for Tier<Item> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        self.inner.witness(index)
    }

    #[inline]
    fn foreach_witness(&self, per_witness: impl FnMut(u64, Hash)) {
        self.inner.foreach_witness(per_witness)
    }
}

impl<Item: GetHash + ForgetOwned> ForgetOwned for Tier<Item> {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        let (inner, forgotten) = self.inner.forget_owned(index);
        (inner.map(|inner| Tier { inner }), forgotten)
    }
}

impl<Item: Complete> From<frontier::Tier<Item::Focus>> for Insert<Tier<Item>> {
    fn from(frontier: frontier::Tier<Item::Focus>) -> Self {
        frontier.finalize_owned()
    }
}
