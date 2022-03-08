use crate::{GetHash, Hash, Height};

use super::super::active;

/// A complete, witnessed leaf of a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leaf<T>(T);

impl<T> Leaf<T> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: T) -> Self {
        Self(item)
    }
}

impl<T: GetHash> GetHash for Leaf<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.0.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.0.cached_hash()
    }
}

impl<T: Height> Height for Leaf<T> {
    type Height = T::Height;
}

impl<T: crate::Complete> crate::Complete for Leaf<T> {
    type Focus = active::Leaf<<T as crate::Complete>::Focus>;
}
