use crate::prelude::*;

/// The hash of the most-recently-inserted item, stored at the tip of the frontier.
#[derive(Debug, Clone, Copy, Derivative, Serialize, Deserialize)]
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
    fn forget(&mut self, _forgotten: Forgotten, index: impl Into<u64>) -> bool {
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

impl Any for Item {
    fn kind(&self) -> Kind {
        Kind::Rightmost(self.item.keep().map(|(commitment, _)| commitment))
    }

    fn global_position(&self) -> Option<u64> {
        <Self as GetPosition>::position(&self)
    }

    fn children(&self) -> Vec<(Insert<Child>, Forgotten)> {
        vec![]
    }
}
