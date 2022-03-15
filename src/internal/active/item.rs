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
    keep: bool,
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
            keep: true,
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
        if self.keep {
            Insert::Keep(complete::Item::new(self.hash))
        } else {
            Insert::Hash(self.hash)
        }
    }
}

impl Witness for Item {
    type Item = Hash;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        if self.keep && index.into() == 0 {
            Some((path::Leaf, self.hash))
        } else {
            None
        }
    }
}

impl Forget for Item {
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        if index.into() == 0 {
            self.keep = false;
            true
        } else {
            false
        }
    }
}
