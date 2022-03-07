use crate::{Finalize, Full, GetHash, Hash, HashOr, Height};

pub struct Active<T> {
    item: HashOr<T>,
}

impl<Item: GetHash> GetHash for Active<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        match self.item {
            HashOr::Hash(hash) => hash,
            HashOr::Item(ref item) => item.hash(),
        }
    }
}

impl<Item: Height> Height for Active<Item> {
    type Height = Item::Height;
}

impl<Item: crate::Finalize> crate::Active for Active<Item> {
    type Item = Item;

    #[inline]
    fn singleton(item: HashOr<Self::Item>) -> Self {
        Self { item }
    }

    #[inline]
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        match self.item {
            HashOr::Hash(_) => None,
            HashOr::Item(ref mut item) => Some(f(item)),
        }
    }

    #[inline]
    fn insert(self, item: HashOr<Self::Item>) -> Result<Self, Full<Self::Item, Self::Complete>> {
        Err(Full {
            item,
            complete: self.finalize(),
        })
    }
}

impl<Item: crate::Finalize> crate::Finalize for Active<Item> {
    type Complete = super::Complete<<Item as crate::Finalize>::Complete>;

    #[inline]
    fn finalize(self) -> HashOr<Self::Complete> {
        match self.item {
            HashOr::Hash(hash) => HashOr::Hash(hash),
            HashOr::Item(item) => match item.finalize() {
                HashOr::Hash(hash) => HashOr::Hash(hash),
                HashOr::Item(item) => HashOr::Item(super::Complete::new(item)),
            },
        }
    }
}
