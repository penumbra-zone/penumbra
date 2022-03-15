use crate::{
    internal::{
        active,
        height::Zero,
        path::{self, Witness},
    },
    AuthPath, Complete, ForgetOwned, GetHash, Hash, Height, Insert,
};

/// A witnessed hash of a commitment at the true leaf of a complete tree.
#[derive(Clone, Copy, PartialEq, Eq, Derivative)]
#[derivative(Debug = "transparent")]
pub struct Item(Hash);

impl Item {
    /// Create a new `Item` from a [`Hash`](crate::Hash).
    pub fn new(hash: Hash) -> Self {
        Self(hash)
    }
}

impl GetHash for Item {
    #[inline]
    fn hash(&self) -> Hash {
        self.0
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.0)
    }
}

impl Height for Item {
    type Height = Zero;
}

impl Complete for Item {
    type Focus = active::Item;
}

impl Witness for Item {
    type Item = Hash;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        if index.into() == 0 {
            Some((path::Leaf, self.0))
        } else {
            None
        }
    }
}

impl ForgetOwned for Item {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        if index.into() == 0 {
            (Insert::Hash(self.0), true)
        } else {
            (Insert::Keep(self), false)
        }
    }
}
