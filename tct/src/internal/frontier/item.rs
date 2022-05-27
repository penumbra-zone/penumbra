use crate::prelude::*;

/// The hash of the most-recently-inserted item, stored at the tip of the frontier.
#[derive(Debug, Clone, Copy, Derivative, Serialize, Deserialize)]
pub struct Item {
    item: Insert<Hash>,
}

impl From<Commitment> for Item {
    fn from(item: Commitment) -> Self {
        Self {
            item: Insert::Keep(Hash::of(item)),
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
            Insert::Keep(hash) => hash,
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
        self.item.map(complete::Item::new)
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

    const CAPACITY: u64 = 1;
}

impl Forget for Item {
    #[inline]
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        if index.into() == 0 {
            if let Insert::Keep(hash) = self.item {
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

impl Visit for Item {
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output {
        visitor.frontier_item(index, self)
    }
}

impl Traverse for Item {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        traversal.traverse(visitor, output, self, visit::NO_CHILDREN, visit::NO_CHILD);
    }
}
