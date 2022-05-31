use crate::prelude::*;

/// A witnessed hash of a commitment at the true leaf of a complete tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Derivative, Serialize, Deserialize)]
pub struct Item {
    hash: Hash,
    commitment: Commitment,
}

impl Item {
    /// Create a new `Item` from a [`Hash`](struct@Hash).
    pub fn new(hash: Hash, commitment: Commitment) -> Self {
        Self { hash, commitment }
    }
}

impl GetHash for Item {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }
}

impl Height for Item {
    type Height = Zero;
}

impl Complete for Item {
    type Focus = frontier::Item;
}

impl Witness for Item {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        debug_assert_eq!(index.into(), 0, "non-zero index when witnessing leaf");
        Some((path::Leaf, self.hash))
    }
}

impl ForgetOwned for Item {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        debug_assert_eq!(index.into(), 0, "non-zero index when forgetting leaf");
        (Insert::Hash(self.hash), true)
    }
}

impl Any for Item {
    fn place(&self) -> Place {
        Place::Complete
    }

    fn kind(&self) -> Kind {
        Kind::Item
    }

    fn height(&self) -> u8 {
        <Self as Height>::Height::HEIGHT
    }

    fn children(&self) -> Vec<Insert<Child>> {
        vec![]
    }
}
