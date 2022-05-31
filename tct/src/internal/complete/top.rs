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

impl<Item: Height + Any> Any for Top<Item> {
    fn place(&self) -> Place {
        Place::Complete
    }

    fn kind(&self) -> Kind {
        Kind::Top
    }

    fn height(&self) -> u8 {
        <Self as Height>::Height::HEIGHT
    }

    fn children(&self) -> Vec<Insert<Child>> {
        vec![Insert::Keep(Child::new(&self.inner))]
    }
}
