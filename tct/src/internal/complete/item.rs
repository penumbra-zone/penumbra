use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        active,
        height::Zero,
        path::{self, Witness},
    },
    AuthPath, Complete, ForgetOwned, GetHash, Hash, Height, Insert,
};

/// A witnessed hash of a commitment at the true leaf of a complete tree.
#[derive(Clone, Copy, PartialEq, Eq, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Item<Hasher>(Hash<Hasher>);

impl<Hasher> Item<Hasher> {
    /// Create a new `Item` from a [`Hash`](crate::Hash).
    pub fn new(hash: Hash<Hasher>) -> Self {
        Self(hash)
    }
}

impl<Hasher> GetHash<Hasher> for Item<Hasher> {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        self.0
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        Some(self.0)
    }
}

impl<Hasher> Height for Item<Hasher> {
    type Height = Zero;
}

impl<Hasher> Complete<Hasher> for Item<Hasher> {
    type Focus = active::Item<Hasher>;
}

impl<Hasher> Witness<Hasher> for Item<Hasher> {
    type Item = Hash<Hasher>;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Hash<Hasher>)> {
        debug_assert_eq!(index.into(), 0, "non-zero index when witnessing leaf");
        Some((path::Leaf, self.0))
    }
}

impl<Hasher> ForgetOwned<Hasher> for Item<Hasher> {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self, Hasher>, bool) {
        debug_assert_eq!(index.into(), 0, "non-zero index when forgetting leaf");
        (Insert::Hash(self.0), true)
    }
}
