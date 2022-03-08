use crate::{Focus, Full, GetHash, Hash, Height, Insert};

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
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        match self.item {
            Insert::Hash(_) => None,
            Insert::Keep(ref mut item) => Some(f(item)),
        }
    }

    #[inline]
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
