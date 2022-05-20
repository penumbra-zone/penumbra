use super::*;

/// This trait is used as a synonym for `impl FnOnce(Parent) -> Tree`: a closure which will
/// construct a tree, as soon as it is given a reference to that tree's parent.
///
/// We create a builder rather than a tree directly because this allows us to compositionally
/// construct trees with parent references.
pub(super) trait Builder {
    /// Construct the tree, given that tree's parent.
    fn with_parent(self, parent: Parent) -> Tree;
}

impl<T: FnOnce(Parent) -> Tree> Builder for T {
    fn with_parent(self, parent: Parent) -> Tree {
        self(parent)
    }
}

/// Make a leaf builder.
fn leaf(commitment: Insert<Commitment>) -> impl Builder {
    move |parent| Tree {
        inner: Arc::new(Inner::new(parent, 0, Node::Leaf(Leaf { commitment }))),
    }
}

/// Make a node builder.
fn node(height: u8, children: impl IntoIterator<Item = impl Builder>) -> impl Builder {
    move |parent| Tree {
        inner: Arc::new_cyclic(|this| {
            let children: Vec<_> = children
                .into_iter()
                .map(|child| child.with_parent(this.clone()))
                .collect();
            assert!(
                !children.is_empty() && children.len() <= 4,
                "nodes must have between 1 and 4 children"
            );

            Inner::new(parent, height, Node::Internal(Internal { children }))
        }),
    }
}

/// Functions for constructing tiers of different shapes.
mod tier {
    use super::*;

    /// Make a builder for an empty tier.
    pub(super) fn empty(parent: Parent, height: u8) -> Tree {
        Tree {
            inner: Arc::new(Inner::new(parent, height, Node::Tier(Tier { root: None }))),
        }
    }

    /// Make a builder for a non-empty tier.
    fn non_empty(parent: Parent, height: u8, contents: Insert<impl Builder>) -> Tree {
        Tree {
            inner: Arc::new_cyclic(|this| {
                Inner::new(
                    parent,
                    height,
                    Node::Tier(Tier {
                        root: Some(contents.map(|contents| contents.with_parent(this.clone()))),
                    }),
                )
            }),
        }
    }

    /// Make a builder for a tier containing some subtree.
    pub(super) fn containing(parent: Parent, height: u8, contents: impl Builder) -> Tree {
        non_empty(parent, height, Insert::Keep(contents))
    }

    /// Make a builder for a tier summarized by some hash value.
    pub(super) fn hashed(parent: Parent, height: u8, hash: Hash) -> Tree {
        non_empty(parent, height, Insert::Hash::<fn(Parent) -> Tree>(hash))
    }
}

/// Starting at some base height, consolidate the builders in the iterator into an 8-node-deep tier.
fn tier(base_height: u8, level_0: Insert<impl IntoIterator<Item = impl Builder>>) -> impl Builder {
    fn level(height: u8, mut level: List<impl Builder>) -> List<impl Builder> {
        let mut next_level = List::new();

        while !level.is_empty() {
            let mut children = List::new();

            for _ in 0..4 {
                if let Some(child) = level.pop_front() {
                    children.push_back(child);
                }
            }

            // We always keep nodes during construction; pruning unneeded nodes is only possible
            // after hashes are computed, but this can't be done until the full tree is constructed.
            next_level.push_back(node(height + 1, children));
        }

        next_level
    }

    move |parent| {
        let level_0 = match level_0 {
            Insert::Hash(hash) => return tier::hashed(parent, base_height + 8, hash),
            Insert::Keep(level_0) => level_0,
        };

        let level_0: List<_> = level_0.into_iter().collect();

        if level_0.is_empty() {
            return tier::empty(parent, base_height + 8);
        }

        let level_1 = level(base_height, level_0);
        let level_2 = level(base_height + 1, level_1);
        let level_3 = level(base_height + 2, level_2);
        let level_4 = level(base_height + 3, level_3);
        let level_5 = level(base_height + 4, level_4);
        let level_6 = level(base_height + 5, level_5);
        let level_7 = level(base_height + 6, level_6);
        let level_8 = level(base_height + 7, level_7);

        assert_eq!(
            level_8.len(),
            1,
            "tiers must contain less than 4^8 elements"
        );

        let root = level_8.into_iter().next().unwrap();

        tier::containing(parent, base_height + 8, root)
    }
}

/// Build a block from an iterator of commitments.
pub(super) fn block(block: Insert<impl IntoIterator<Item = Insert<Commitment>>>) -> impl Builder {
    tier(0, block.map(|leaves| leaves.into_iter().map(self::leaf)))
}

/// Build an epoch from an iterator of blocks.
pub(super) fn epoch(
    epoch: Insert<impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>>,
) -> impl Builder {
    tier(8, epoch.map(|blocks| blocks.into_iter().map(self::block)))
}

/// Build an eternity from an iterator of epochs.
pub(super) fn eternity(
    eternity: impl IntoIterator<
        Item = Insert<
            impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
        >,
    >,
) -> impl Builder {
    tier(16, Insert::Keep(eternity.into_iter().map(self::epoch)))
}
