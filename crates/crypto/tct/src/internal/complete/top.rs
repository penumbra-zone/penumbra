use crate::prelude::*;

use complete::Nested;

/// A complete top-level tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Top<Item: GetHash + Height + Clone> {
    pub(in super::super) inner: Nested<Item>,
}

impl<Item: GetHash + Height + Clone> Height for Top<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: GetHash + Height + Clone> GetHash for Top<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: GetHash + Height + Clone> From<complete::Tier<Item>> for Top<Item> {
    fn from(tier: complete::Tier<Item>) -> Self {
        Top { inner: tier.inner }
    }
}

impl<Item: Height + GetHash + Clone> GetPosition for Top<Item> {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<'tree, Item: Height + structure::Any<'tree> + Clone> structure::Any<'tree> for Top<Item> {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn forgotten(&self) -> Forgotten {
        (&self.inner as &dyn structure::Any).forgotten()
    }

    fn children(&'tree self) -> Vec<HashOrNode<'tree>> {
        (&self.inner as &dyn structure::Any).children()
    }
}

impl<Item: GetHash + Height + OutOfOrderOwned + Clone> OutOfOrderOwned for Top<Item> {
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: StateCommitment,
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

impl<Item: GetHash + UncheckedSetHash + Clone> UncheckedSetHash for Top<Item> {
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        self.inner.unchecked_set_hash(index, height, hash)
    }

    fn finish_initialize(&mut self) {
        self.inner.finish_initialize()
    }
}
