use archery::SharedPointerKind;

use crate::prelude::*;

use complete::Nested;

/// A complete top-level tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug)]
pub struct Top<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> {
    pub(in super::super) inner: Nested<Item, RefKind>,
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> Height for Top<Item, RefKind> {
    type Height = <Nested<Item, RefKind> as Height>::Height;
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> GetHash for Top<Item, RefKind> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: GetHash + Height + Clone, RefKind: SharedPointerKind> From<complete::Tier<Item, RefKind>>
    for Top<Item, RefKind>
{
    fn from(tier: complete::Tier<Item, RefKind>) -> Self {
        Top { inner: tier.inner }
    }
}

impl<Item: Height + GetHash + Clone, RefKind: SharedPointerKind> GetPosition
    for Top<Item, RefKind>
{
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<Item: Height + structure::Any<RefKind> + Clone, RefKind: SharedPointerKind>
    structure::Any<RefKind> for Top<Item, RefKind>
{
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        (&self.inner as &dyn structure::Any<RefKind>).forgotten()
    }

    fn children(&self) -> Vec<Node<RefKind>> {
        (&self.inner as &dyn structure::Any<RefKind>).children()
    }
}

impl<Item: GetHash + Height + OutOfOrderOwned + Clone, RefKind: SharedPointerKind> OutOfOrderOwned
    for Top<Item, RefKind>
{
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: Commitment,
    ) -> Self {
        Top {
            inner: Nested::uninitialized_out_of_order_insert_commitment_owned(
                this.map(|tier| tier.inner),
                index,
                commitment,
            ),
        }
    }
}

impl<Item: GetHash + UncheckedSetHash + Clone, RefKind: SharedPointerKind> UncheckedSetHash
    for Top<Item, RefKind>
{
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        self.inner.unchecked_set_hash(index, height, hash)
    }

    fn finish_initialize(&mut self) {
        self.inner.finish_initialize()
    }
}
