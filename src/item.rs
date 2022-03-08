//! Items at the leaves of a tree, paired with a lazily-computed hash.

use std::cell::Cell;

use crate::{internal::height::Zero, GetHash, Hash, Insert};

/// Both a hash and the item hashed, with the hash computed lazily, to be used for inserting into a
/// tree. This implements [`Focus`](crate::Focus) and thus can be used as the item of a tree.
///
/// If you don't want to store actual items at the leaves of a tree but rather just store their
/// hashes, use [`struct@Hash`] directly as the item of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<T> {
    // TODO: replace with `OptionHash` optimization?
    hash: Cell<Option<Hash>>,
    item: T,
}

impl<T> Item<T> {
    /// Create a new [`Item`] from the given value.
    pub fn new(item: T) -> Self {
        Self {
            hash: Cell::new(None),
            item,
        }
    }
}

impl<T> From<T> for Item<T> {
    fn from(item: T) -> Self {
        Self::new(item)
    }
}

impl<T> AsRef<T> for Item<T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<T: GetHash> GetHash for Item<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash.get().unwrap_or_else(|| {
            let hash = self.item.hash();
            self.hash.set(Some(hash));
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get()
    }
}

impl<T> crate::Height for Item<T> {
    type Height = Zero;
}

impl<T: GetHash> crate::Focus for Item<T> {
    type Complete = Self;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        Insert::Keep(self)
    }
}

impl<T: GetHash> crate::Complete for Item<T> {
    type Focus = Self;
}
