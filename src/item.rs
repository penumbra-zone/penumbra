use std::cell::Cell;

use crate::{height::Z, GetHash, Hash, HashOr};

/// Both a hash and the item hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<T> {
    // TODO: replace with `OptionHash` optimization?
    hash: Cell<Option<Hash>>,
    item: T,
}

impl<T> Item<T> {
    pub fn new(item: T) -> Self {
        Self {
            hash: Cell::new(None),
            item,
        }
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
}

impl<T> crate::Height for Item<T> {
    type Height = Z;
}

impl<T: GetHash> crate::Focus for Item<T> {
    type Complete = Self;

    #[inline]
    fn finalize(self) -> HashOr<Self::Complete> {
        HashOr::Item(self)
    }
}

impl<T: GetHash> crate::Complete for Item<T> {
    type Active = Self;
}
