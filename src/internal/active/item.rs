use crate::Commitment;

use crate::{
    internal::{
        active::Forget,
        complete,
        height::Zero,
        path::{self, Witness},
    },
    AuthPath, Focus, GetHash, Hash, Height, Insert,
};

/// The hash of the most-recently-inserted item, stored at the tip of the active path.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct Item {
    hash: Hash,
}

impl PartialEq<complete::Item> for Item {
    fn eq(&self, other: &complete::Item) -> bool {
        self.hash == other.hash()
    }
}

impl Item {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: Commitment) -> Self {
        Self {
            hash: Hash::of(item),
        }
    }
}

impl From<Commitment> for Item {
    fn from(item: Commitment) -> Self {
        Self::new(item)
    }
}

impl From<Item> for Hash {
    fn from(item: Item) -> Self {
        item.hash
    }
}

impl GetHash for Item {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }
}

impl Height for Item {
    type Height = Zero;
}

impl Focus for Item {
    type Complete = complete::Item;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        Insert::Keep(complete::Item::new(self.hash))
    }
}

impl Witness for Item {
    type Item = Hash;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        debug_assert_eq!(index.into(), 0, "non-zero index when witnessing leaf");
        Some((path::Leaf, self.hash))
    }
}

impl Forget for Item {
    fn forget(&mut self, _index: impl Into<u64>) -> bool {
        unreachable!("active items can not be forgotten directly")
    }
}
