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
    fn forget_owned(
        self,
        _forgotten: Option<Forgotten>,
        index: impl Into<u64>,
    ) -> (Insert<Self>, bool) {
        debug_assert_eq!(index.into(), 0, "non-zero index when forgetting leaf");
        (Insert::Hash(self.hash), true)
    }
}

impl GetPosition for Item {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<'tree> structure::Any<'tree> for Item {
    fn kind(&self) -> Kind {
        Kind::Leaf {
            commitment: Some(self.commitment),
        }
    }

    fn forgotten(&self) -> Forgotten {
        Forgotten::default()
    }

    fn children(&self) -> Vec<HashOrNode<'tree>> {
        vec![]
    }
}

impl OutOfOrderOwned for Item {
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: Commitment,
    ) -> Self {
        if index != 0 {
            panic!("non-zero index when inserting commitment");
        }
        let hash = match this {
            Insert::Keep(Item { hash, .. }) => hash,
            Insert::Hash(hash) => hash,
        };
        Item { hash, commitment }
    }
}

impl UncheckedSetHash for Item {
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        if index != 0 {
            panic!("non-zero index when setting hash");
        }
        if height != 0 {
            panic!("non-zero height when setting hash");
        }
        self.hash = hash;
    }

    fn finish_initialize(&mut self) {
        if self.hash.is_uninitialized() {
            self.hash = Hash::of(self.commitment);
        }
    }
}
