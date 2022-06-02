use crate::prelude::*;

type N<Child> = super::super::complete::Node<Child>;
type L<Item> = super::super::complete::Leaf<Item>;

/// An eight-deep complete tree with the given item at each leaf.
pub type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// Count the levels:    1 2 3 4 5 6 7 8

/// A complete tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tier<Item: GetHash + Height> {
    pub(in super::super) inner: Nested<Item>,
}

impl<Item: GetHash + Height> Height for Tier<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: GetHash + Height> GetHash for Tier<Item> {
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
}

impl<Item: GetHash + ForgetOwned> ForgetOwned for Tier<Item> {
    fn forget_owned(
        self,
        forgotten: Option<Forgotten>,
        index: impl Into<u64>,
    ) -> (Insert<Self>, bool) {
        let (inner, forgotten) = self.inner.forget_owned(forgotten, index);
        (inner.map(|inner| Tier { inner }), forgotten)
    }
}

impl<Item: Complete> From<frontier::Tier<Item::Focus>> for Insert<Tier<Item>> {
    fn from(frontier: frontier::Tier<Item::Focus>) -> Self {
        frontier.finalize_owned()
    }
}

impl<Item: GetHash + Height> GetPosition for Tier<Item> {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<Item: Height + structure::Node> structure::Node for Tier<Item> {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn global_position(&self) -> Option<u64> {
        <Self as GetPosition>::position(self)
    }

    fn forgotten(&self) -> Forgotten {
        structure::Node::forgotten(&self.inner)
    }

    fn children(&self) -> Vec<Child> {
        (&self.inner as &dyn structure::Node).children()
    }
}
