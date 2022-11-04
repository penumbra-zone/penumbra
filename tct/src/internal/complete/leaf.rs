use archery::SharedPointerKind;

use crate::prelude::*;

/// A complete, witnessed leaf of a tree.
#[derive(Clone, Copy, Derivative)]
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
}

impl<Item: ForgetOwned> ForgetOwned for Leaf<Item> {
    fn forget_owned(
        self,
        forgotten: Option<Forgotten>,
        index: impl Into<u64>,
    ) -> (Insert<Self>, bool) {
        let (item, forgotten) = self.0.forget_owned(forgotten, index);
        (item.map(Leaf), forgotten)
    }
}

impl<Item> GetPosition for Leaf<Item> {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<Item: Height + structure::Any<RefKind>, RefKind: SharedPointerKind> structure::Any<RefKind>
    for Leaf<Item>
{
    fn kind(&self) -> Kind {
        self.0.kind()
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        self.0.forgotten()
    }

    fn children(&self) -> Vec<Node<RefKind>> {
        self.0.children()
    }
}

impl<Item: OutOfOrderOwned> OutOfOrderOwned for Leaf<Item> {
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: Commitment,
    ) -> Self {
        Leaf(Item::uninitialized_out_of_order_insert_commitment_owned(
            this.map(|leaf| leaf.0),
            index,
            commitment,
        ))
    }
}

impl<Item: UncheckedSetHash> UncheckedSetHash for Leaf<Item> {
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        self.0.unchecked_set_hash(index, height, hash)
    }

    fn finish_initialize(&mut self) {
        self.0.finish_initialize()
    }
}
