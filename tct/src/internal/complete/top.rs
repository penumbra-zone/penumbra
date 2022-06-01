use crate::prelude::*;

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

impl<Item: Height + GetHash + ForgetForgotten> ForgetForgotten for Top<Item> {
    fn forget_forgotten(&mut self) {
        self.inner.forget_forgotten()
    }
}

impl<Item: Height + Any> Any for Top<Item> {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn global_position(&self) -> Option<u64> {
        <Self as GetPosition>::position(self)
    }

    fn children(&self) -> Vec<(Forgotten, Insert<Child>)> {
        (&self.inner as &dyn Any).children()
    }
}
