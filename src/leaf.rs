use crate::{Active, Complete, GetHash, Hash, Height};

// TODO: separate leaves into active and complete

pub struct Leaf<T, const BASE_HEIGHT: usize> {
    item: T,
    witnessed: bool,
}

impl<T: GetHash, const BASE_HEIGHT: usize> GetHash for Leaf<T, BASE_HEIGHT> {
    fn hash(&self) -> Hash {
        self.item.hash()
    }
}

impl<T, const BASE_HEIGHT: usize> Height for Leaf<T, BASE_HEIGHT> {
    const HEIGHT: usize = BASE_HEIGHT;
}

impl<T, const BASE_HEIGHT: usize> Active for Leaf<T, BASE_HEIGHT> {
    type Item = T;
    type Complete = Leaf<T, BASE_HEIGHT>;

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
        Err((item, self))
    }

    #[inline]
    fn complete(self) -> Self::Complete {
        self
    }
}

impl<T, const BASE_HEIGHT: usize> Complete for Leaf<T, BASE_HEIGHT> {
    type Active = Leaf<T, BASE_HEIGHT>;

    fn witnessed(&self) -> bool {
        self.witnessed
    }
}
