use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        active::Forget,
        complete, hash,
        height::Zero,
        path::{self, Witness},
    },
    AuthPath, Commitment, Focus, GetHash, Hash, Height, Insert,
};

/// The hash of the most-recently-inserted item, stored at the tip of the active path.
#[derive(Debug, Clone, Copy, Derivative, Serialize, Deserialize)]
#[derivative(PartialEq, Eq)]
pub struct Item<Hasher> {
    hash: Hash<Hasher>,
}

impl<Hasher> PartialEq<complete::Item<Hasher>> for Item<Hasher> {
    fn eq(&self, other: &complete::Item<Hasher>) -> bool {
        self.hash == other.hash()
    }
}

impl<Hasher: hash::Hasher> Item<Hasher> {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: Commitment) -> Self {
        Self {
            hash: Hash::of(item),
        }
    }
}

impl<Hasher: hash::Hasher> From<Commitment> for Item<Hasher> {
    fn from(item: Commitment) -> Self {
        Self::new(item)
    }
}

impl<Hasher> From<Item<Hasher>> for Hash<Hasher> {
    fn from(item: Item<Hasher>) -> Self {
        item.hash
    }
}

impl<Hasher> GetHash<Hasher> for Item<Hasher> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        Some(self.hash)
    }
}

impl<Hasher> Height for Item<Hasher> {
    type Height = Zero;
}

impl<Hasher> Focus<Hasher> for Item<Hasher> {
    type Complete = complete::Item<Hasher>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete, Hasher> {
        Insert::Keep(complete::Item::new(self.hash))
    }
}

impl<Hasher> Witness<Hasher> for Item<Hasher> {
    type Item = Hash<Hasher>;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Self::Item)> {
        debug_assert_eq!(index.into(), 0, "non-zero index when witnessing leaf");
        Some((path::Leaf, self.hash))
    }
}

impl<Hasher> Forget for Item<Hasher> {
    fn forget(&mut self, _index: impl Into<u64>) -> bool {
        unreachable!("active items can not be forgotten directly")
    }
}
