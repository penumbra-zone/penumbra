//! Items at the leaves of a tree, paired with a lazily-computed hash.

use std::cell::Cell;

use poseidon377::Fq;

use crate::{internal::height::Zero, GetHash, Hash, Height, Insert};

/// Both a hash and the item hashed, with the hash computed lazily, to be used when inserting into a
/// tree.
///
/// If you don't want to store actual items at the leaves of a tree but rather just store their
/// hashes, use [`struct@Hash`] directly as the item of the tree.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct Item {
    #[derivative(PartialEq = "ignore")]
    hash: Cell<Option<Hash>>,
    item: Fq,
}

impl Item {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: Fq) -> Self {
        Self {
            hash: Cell::new(None),
            item,
        }
    }
}

impl From<Fq> for Item {
    fn from(item: Fq) -> Self {
        Self::new(item)
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
        self.hash.get().unwrap_or_else(|| {
            let hash = Hash::item(self.item);
            self.hash.set(Some(hash));
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get()
    }
}

impl Height for Item {
    type Height = Zero;
}

impl crate::Focus for Item {
    type Complete = Self;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        Insert::Keep(self)
    }
}

impl crate::Complete for Item {
    type Focus = Self;
}
