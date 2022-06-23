use crate::prelude::*;

mod builder;
pub use builder::Builder;

use complete::Nested;

/// A complete top-level tier of the tiered commitment tree, being an 8-deep sparse quad-tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Top<Item: GetHash + Height> {
    pub(in super::super) inner: Nested<Item>,
}

impl<Item: GetHash + Height> Height for Top<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: GetHash + Height> GetHash for Top<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<Item: GetHash + Height> From<complete::Tier<Item>> for Top<Item> {
    fn from(tier: complete::Tier<Item>) -> Self {
        Top { inner: tier.inner }
    }
}

impl<Item: Height + GetHash> GetPosition for Top<Item> {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<Item: Height + structure::Any> structure::Any for Top<Item> {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        (&self.inner as &dyn structure::Any).forgotten()
    }

    fn children(&self) -> Vec<Node> {
        (&self.inner as &dyn structure::Any).children()
    }
}
