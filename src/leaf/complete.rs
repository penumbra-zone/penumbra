use crate::{GetHash, Hash, Height};

enum Inner<T> {
    Hash(Hash),
    Item(T),
}

pub struct Complete<T, const BASE_HEIGHT: usize> {
    inner: Inner<T>,
}

impl<T, const BASE_HEIGHT: usize> Complete<T, BASE_HEIGHT> {
    pub fn from_hash(hash: Hash) -> Self {
        Self {
            inner: Inner::Hash(hash),
        }
    }

    pub fn from_item(item: T) -> Self {
        Self {
            inner: Inner::Item(item),
        }
    }
}

impl<T: GetHash, const BASE_HEIGHT: usize> GetHash for Complete<T, BASE_HEIGHT> {
    fn hash(&self) -> Hash {
        match &self.inner {
            Inner::Hash(hash) => *hash,
            Inner::Item(item) => item.hash(),
        }
    }
}

impl<T, const BASE_HEIGHT: usize> Height for Complete<T, BASE_HEIGHT> {
    const HEIGHT: usize = BASE_HEIGHT;
}

impl<T: GetHash, const BASE_HEIGHT: usize> crate::Complete for Complete<T, BASE_HEIGHT> {
    type Active = super::Active<T, BASE_HEIGHT>;

    fn witnessed(&self) -> bool {
        matches!(self.inner, Inner::Item(_))
    }
}
