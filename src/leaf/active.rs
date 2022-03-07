use crate::{GetHash, Hash, Height};

pub struct Active<T> {
    item: Result<T, Hash>,
}

impl<Item: GetHash> GetHash for Active<Item> {
    fn hash(&self) -> Hash {
        self.item
            .as_ref()
            .map(|item| item.hash())
            .unwrap_or_else(|hash| *hash)
    }
}

impl<Item: Height> Height for Active<Item> {
    const HEIGHT: usize = Item::HEIGHT;
}

impl<Item: crate::Active> crate::Active for Active<Item> {
    type Item = Item;
    type Complete = super::Complete<<Item as crate::Active>::Complete>;

    #[inline]
    fn singleton(item: Self::Item) -> Self {
        Self { item: Ok(item) }
    }

    #[inline]
    fn alter<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        self.item.as_mut().map(f).ok()
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Result<Self::Complete, Hash>)> {
        Err((item, self.complete()))
    }

    #[inline]
    fn complete(self) -> Result<Self::Complete, Hash> {
        let item = self.item?;
        Ok(super::Complete::new(item.complete()?))
    }
}
