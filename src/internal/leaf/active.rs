use crate::{Focus, Full, GetHash, Hash, Height, Insert};

/// The active (rightmost) leaf in an active tree.
///
/// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to contain
/// the inserted item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Active<T> {
    item: Insert<T>,
}

impl<Item: GetHash> GetHash for Active<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        match self.item {
            Insert::Hash(hash) => hash,
            Insert::Keep(ref item) => item.hash(),
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        match self.item {
            Insert::Hash(hash) => Some(hash),
            Insert::Keep(ref item) => item.cached_hash(),
        }
    }
}

impl<Item: Height> Height for Active<Item> {
    type Height = Item::Height;
}

impl<Item: crate::Focus> crate::Active for Active<Item> {
    type Item = Item;

    #[inline]
    fn singleton(item: Insert<Self::Item>) -> Self {
        Self { item }
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Self::Item>) -> T) -> T {
        f(&mut self.item)
    }

    #[inline]
    fn last(&self) -> &Insert<Self::Item> {
        &self.item
    }

    #[inline]
    /// Insertion into a leaf always fails, causing the tree above it to insert a new leaf to
    /// contain the inserted item.
    fn insert(self, item: Insert<Self::Item>) -> Result<Self, Full<Self>> {
        Err(Full {
            item,
            complete: self.finalize(),
        })
    }
}

impl<Item: crate::Focus> crate::Focus for Active<Item> {
    type Complete = super::Complete<<Item as crate::Focus>::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        match self.item {
            Insert::Hash(hash) => Insert::Hash(hash),
            Insert::Keep(item) => match item.finalize() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(item) => Insert::Keep(super::Complete::new(item)),
            },
        }
    }
}
