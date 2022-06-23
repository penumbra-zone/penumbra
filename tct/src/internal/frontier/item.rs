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

impl structure::Any for Item {
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

    fn children(&self) -> Vec<Node> {
        vec![]
    }
}

/// A builder for the tip of the frontier.
pub struct Builder {
    index: u64,
    inner: Inner,
}

enum Inner {
    Init,
    Witnessed { hash: Option<Hash> },
}

impl Built for Item {
    type Builder = Builder;

    fn build(_global_position: u64, index: u64) -> Self::Builder {
        Builder {
            index,
            inner: Inner::Init,
        }
    }
}

impl Build for Builder {
    type Output = Item;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        use {IResult::*, Inner::*, Instruction::*};

        match (&self.inner, instruction) {
            (Init, Leaf { here }) => Ok(Complete(Hash::new(here).into())),
            (Init, Node { here, .. }) => {
                self.inner = Witnessed {
                    hash: here.map(Hash::new),
                };
                Ok(Incomplete(self))
            }
            (Witnessed { hash: None }, Leaf { here }) => Ok(Complete(Commitment(here).into())),
            (Witnessed { hash: Some(hash) }, Leaf { here }) => Ok(Complete(Item {
                item: Insert::Keep((Commitment(here), *hash)),
            })),
            (Witnessed { .. }, Node { .. }) => Err(HitBottom(self)),
        }
    }

    fn is_started(&self) -> bool {
        !matches!(self.inner, Inner::Init)
    }

    fn index(&self) -> u64 {
        self.index
    }

    fn height(&self) -> u8 {
        0
    }

    fn min_required(&self) -> usize {
        1
    }
}
