//! Items at the leaves of a tree, paired with a lazily-computed hash.

use poseidon377::Fq;

use crate::{
    internal::height::Zero, AuthPath, Complete, Focus, GetHash, Hash, Height, Insert, Witness,
};

/// Both a hash and the item hashed, used internally when inserting into a tree.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct Item {
    #[derivative(PartialEq = "ignore")]
    hash: Hash,
    item: Fq,
}

impl Item {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: Fq) -> Self {
        Self {
            hash: Hash::of(item),
            item,
        }
    }
}

impl From<Fq> for Item {
    fn from(item: Fq) -> Self {
        Self::new(item)
    }
}

impl From<Item> for Fq {
    fn from(item: Item) -> Self {
        item.item
    }
}

impl From<Item> for Hash {
    fn from(item: Item) -> Self {
        item.hash
    }
}

impl AsRef<Fq> for Item {
    fn as_ref(&self) -> &Fq {
        &self.item
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
    type Complete = Self;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        Insert::Keep(self)
    }
}

impl Complete for Item {
    type Focus = Self;
}

impl Witness for Item {
    type Item = Fq;

    fn witness(&self, index: u64) -> Option<(AuthPath<Self>, Self::Item)> {
        self.hash
            .witness(index)
            .map(|(auth_path, _)| (auth_path, self.item))
    }
}
