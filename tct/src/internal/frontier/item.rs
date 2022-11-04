use archery::SharedPointerKind;

use crate::prelude::*;

/// The hash of the most-recently-inserted item, stored at the tip of the frontier.
#[derive(Debug, Clone, Copy, Derivative)]
pub struct Item {
    item: Insert<(Commitment, Hash)>,
}

impl From<Commitment> for Item {
    fn from(commitment: Commitment) -> Self {
        Self {
            item: Insert::Keep((commitment, Hash::of(commitment))),
        }
    }
}

impl From<Hash> for Item {
    fn from(hash: Hash) -> Self {
        Self {
            item: Insert::Hash(hash),
        }
    }
}

impl GetHash for Item {
    #[inline]
    fn hash(&self) -> Hash {
        match self.item {
            Insert::Hash(hash) => hash,
            Insert::Keep((_, hash)) => hash,
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash())
    }
}

impl Height for Item {
    type Height = Zero;
}

impl Focus for Item {
    type Complete = complete::Item;

    #[inline]
    fn finalize_owned(self) -> Insert<Self::Complete> {
        self.item
            .map(|(commitment, hash)| complete::Item::new(hash, commitment))
    }
}

impl Witness for Item {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        debug_assert_eq!(index.into(), 0, "non-zero index when witnessing leaf");
        Some((path::Leaf, self.hash()))
    }
}

impl GetPosition for Item {
    #[inline]
    fn position(&self) -> Option<u64> {
        None
    }
}

impl Forget for Item {
    #[inline]
    fn forget(&mut self, _forgotten: Option<Forgotten>, index: impl Into<u64>) -> bool {
        if index.into() == 0 {
            if let Insert::Keep((_, hash)) = self.item {
                self.item = Insert::Hash(hash);
                true
            } else {
                false
            }
        } else {
            panic!("non-zero index when forgetting item");
        }
    }
}

impl<RefKind: SharedPointerKind> structure::Any<RefKind> for Item {
    fn kind(&self) -> Kind {
        Kind::Leaf {
            commitment: self.item.keep().map(|(commitment, _)| commitment),
        }
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        Forgotten::default()
    }

    fn children(&self) -> Vec<Node<RefKind>> {
        vec![]
    }
}

impl OutOfOrder for Item {
    fn uninitialized(_position: Option<u64>, _forgotten: Forgotten) -> Self {
        Self {
            item: Insert::Hash(Hash::uninitialized()),
        }
    }

    fn uninitialized_out_of_order_insert_commitment(&mut self, index: u64, commitment: Commitment) {
        if index == 0 {
            let hash = match self.item {
                Insert::Keep((_drop_old_commitment, hash)) => hash,
                Insert::Hash(hash) => hash,
            };
            self.item = Insert::Keep((commitment, hash));
        } else {
            panic!("non-zero index when inserting commitment");
        }
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
        self.item = match self.item {
            Insert::Keep((commitment, _drop_old_hash)) => Insert::Keep((commitment, hash)),
            Insert::Hash(_drop_old_hash) => Insert::Hash(hash),
        }
    }

    fn finish_initialize(&mut self) {
        match self.item {
            Insert::Keep((commitment, ref mut hash)) => {
                if hash.is_uninitialized() {
                    *hash = Hash::of(commitment);
                }
            }
            Insert::Hash(ref mut hash) => {
                if hash.is_uninitialized() {
                    // An uninitialized frontier hash should be set to the zero hash, which is the
                    // empty hash for the frontier
                    *hash = Hash::zero();
                }
            }
        }
    }
}
