use crate::{GetHash, Hash, Height};

pub struct Active<T, const BASE_HEIGHT: usize> {
    item: T,
    witnessed: bool,
}

impl<T: GetHash, const BASE_HEIGHT: usize> GetHash for Active<T, BASE_HEIGHT> {
    fn hash(&self) -> Hash {
        self.item.hash()
    }
}

impl<T, const BASE_HEIGHT: usize> Height for Active<T, BASE_HEIGHT> {
    const HEIGHT: usize = BASE_HEIGHT;
}

impl<T: GetHash, const BASE_HEIGHT: usize> crate::Active for Active<T, BASE_HEIGHT> {
    type Item = T;
    type Complete = super::Complete<T, BASE_HEIGHT>;

    #[inline]
    fn singleton(item: Self::Item) -> Self {
        Self {
            item,
            witnessed: false,
        }
    }

    #[inline]
    fn witness(&mut self) {
        self.witnessed = true;
    }

    #[inline]
    fn alter(&mut self, f: impl FnOnce(&mut Self::Item)) {
        f(&mut self.item);
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Self::Complete)> {
        Err((item, self.complete()))
    }

    #[inline]
    fn complete(self) -> Self::Complete {
        if self.witnessed {
            super::Complete::from_item(self.item)
        } else {
            super::Complete::from_hash(self.hash())
        }
    }
}
