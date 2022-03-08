use crate::{GetHash, Hash, Height};

type N<Child> = super::super::node::Complete<Child>;
type L<Item> = super::super::leaf::Complete<Item>;

/// An eight-deep complete tree with the given item at each leaf.
pub(super) type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// You can count the levels:   1 2 3 4 5 6 7 8

/// A complete tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complete<Item> {
    pub(super) inner: Nested<Item>,
}

impl<Item: Height> Height for Complete<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: GetHash> GetHash for Complete<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: crate::Complete> crate::Complete for Complete<Item> {
    type Focus = super::Active<Item::Focus>;
}
