use crate::{GetHash, Hash, Height};

/// A complete, witnessed leaf of a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Complete<T>(T);

impl<T> Complete<T> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: T) -> Self {
        Self(item)
    }
}

impl<T: GetHash> GetHash for Complete<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.0.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.0.cached_hash()
    }
}

impl<T: Height> Height for Complete<T> {
    type Height = T::Height;
}

impl<T: crate::Complete> crate::Complete for Complete<T> {
    type Focus = super::Active<<T as crate::Complete>::Focus>;
}
