//! Items at the leaves of a tree, paired with a lazily-computed hash.

use poseidon377::Fq;

use crate::{
    internal::height::Zero, AuthPath, Focus, Forget, GetHash, Hash, Height, Insert, Witness,
};

/// Both a hash and the item hashed, used internally when inserting into a tree.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct Item {
    keep: bool,
    hash: Hash,
}

impl PartialEq<Hash> for Item {
    fn eq(&self, hash: &Hash) -> bool {
        self.hash == *hash
    }
}

impl Item {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: Fq) -> Self {
        Self {
            hash: Hash::of(item),
            keep: true,
        }
    }
}

impl From<Fq> for Item {
    fn from(item: Fq) -> Self {
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
    type Complete = Hash;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        if self.keep {
            Insert::Keep(self.hash)
        } else {
            Insert::Hash(self.hash)
        }
    }
}

impl Witness for Item {
    type Item = Hash;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        self.hash
            .witness(index)
            .map(|(auth_path, _)| (auth_path, self.hash))
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
