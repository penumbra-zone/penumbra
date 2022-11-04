use archery::SharedPointerKind;

use crate::prelude::*;

type N<Child, RefKind> = super::super::complete::Node<Child, RefKind>;
type L<Item> = super::super::complete::Leaf<Item>;

/// An eight-deep complete tree with the given item at each leaf.
pub type Nested<Item, R> = N<N<N<N<N<N<N<N<L<Item>, R>, R>, R>, R>, R>, R>, R>, R>;
// Count the levels:       1 2 3 4 5 6 7 8

/// A complete tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Derivative, Debug)]
#[derivative(Clone(bound = "Item: Clone"))]
pub struct Tier<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> {
    pub(in super::super) inner: Nested<Item, RefKind>,
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> Height for Tier<Item, RefKind> {
    type Height = <Nested<Item, RefKind> as Height>::Height;
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> GetHash for Tier<Item, RefKind> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: Complete + Clone, RefKind: SharedPointerKind> Complete for Tier<Item, RefKind>
where
    Item::Focus: Clone,
{
    type Focus = frontier::Tier<Item::Focus, RefKind>;
}

impl<Item: GetHash + Witness + Clone, RefKind: SharedPointerKind> Witness for Tier<Item, RefKind> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        self.inner.witness(index)
    }
}

impl<Item: GetHash + ForgetOwned + Clone, RefKind: SharedPointerKind> ForgetOwned
    for Tier<Item, RefKind>
{
    fn forget_owned(
        self,
        forgotten: Option<Forgotten>,
        index: impl Into<u64>,
    ) -> (Insert<Self>, bool) {
        let (inner, forgotten) = self.inner.forget_owned(forgotten, index);
        (inner.map(|inner| Tier { inner }), forgotten)
    }
}

impl<Item: Complete + Clone, RefKind: SharedPointerKind> From<frontier::Tier<Item::Focus, RefKind>>
    for Insert<Tier<Item, RefKind>>
where
    Item::Focus: Clone,
{
    fn from(frontier: frontier::Tier<Item::Focus, RefKind>) -> Self {
        frontier.finalize_owned()
    }
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> GetPosition
    for Tier<Item, RefKind>
{
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<Item: Height + structure::Any<RefKind> + Clone, RefKind: SharedPointerKind>
    structure::Any<RefKind> for Tier<Item, RefKind>
{
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        structure::Any::forgotten(&self.inner)
    }

    fn children(&self) -> Vec<Node<RefKind>> {
        (&self.inner as &dyn structure::Any<RefKind>).children()
    }
}

impl<Item: GetHash + Height + OutOfOrderOwned + Clone, RefKind: SharedPointerKind> OutOfOrderOwned
    for Tier<Item, RefKind>
{
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: Commitment,
    ) -> Self {
        Tier {
            inner: Nested::uninitialized_out_of_order_insert_commitment_owned(
                this.map(|tier| tier.inner),
                index,
                commitment,
            ),
        }
    }
}

impl<Item: GetHash + UncheckedSetHash + Clone, RefKind: SharedPointerKind> UncheckedSetHash
    for Tier<Item, RefKind>
{
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        self.inner.unchecked_set_hash(index, height, hash)
    }

    fn finish_initialize(&mut self) {
        self.inner.finish_initialize()
    }
}
