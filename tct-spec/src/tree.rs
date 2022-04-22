use std::{
    collections::VecDeque as List,
    sync::{Arc, Weak},
};

use crate::*;

pub struct Tree {
    inner: Arc<Wrapped>,
}

type Parent = Weak<Wrapped>;

struct Wrapped {
    parent: Parent,
    height: u8,
    hash: CachedHash,
    inner: Inner,
}

impl Wrapped {
    pub fn with_parent_and_height(parent: Parent, height: u8, inner: Inner) -> Self {
        Self {
            parent,
            height,
            hash: CachedHash::default(),
            inner,
        }
    }
}

enum Inner {
    Leaf(Leaf),
    Node(Node),
    Tier(Tier),
}

pub struct Leaf {
    pub commitment: Insert<Commitment>,
}

pub struct Node {
    pub children: Vec<Tree>,
}

pub struct Tier {
    pub root: Option<Insert<Tree>>,
}

mod build;
use build::Builder;

impl Tree {
    fn is(&self, other: &Tree) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    fn parent(&self) -> Option<Tree> {
        self.inner.parent.upgrade().map(|inner| Tree { inner })
    }

    fn inner(&self) -> &Inner {
        &self.inner.inner
    }

    pub fn from_block(block: impl IntoIterator<Item = Insert<Commitment>>) -> Tree {
        build::block(Insert::Keep(block)).with_parent(Parent::new())
    }

    pub fn from_epoch(
        epoch: impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
    ) -> Tree {
        build::epoch(Insert::Keep(epoch)).with_parent(Parent::new())
    }

    pub fn from_eternity(
        eternity: impl IntoIterator<
            Item = Insert<
                impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
            >,
        >,
    ) -> Tree {
        build::eternity(eternity).with_parent(Parent::new())
    }

    pub fn as_leaf(&self) -> &Leaf {
        if let Inner::Leaf(leaf) = self.inner() {
            leaf
        } else {
            panic!("expected tree to be a leaf")
        }
    }

    pub fn as_node(&self) -> &Node {
        if let Inner::Node(node) = self.inner() {
            node
        } else {
            panic!("expected tree to be a node")
        }
    }

    pub fn as_tier(&self) -> &Tier {
        if let Inner::Tier(tier) = self.inner() {
            tier
        } else {
            panic!("expected tree to be a tier")
        }
    }

    fn is_frontier(&self) -> bool {
        if let Some(parent) = self.parent() {
            match parent.inner() {
                Inner::Tier(_) => parent.is_frontier(),
                Inner::Node(node) => node.children.last().unwrap().is(self) && parent.is_frontier(),
                Inner::Leaf(_) => unreachable!("the parent of a tree can never be a leaf"),
            }
        } else {
            true
        }
    }

    fn height(&self) -> u8 {
        self.inner.height
    }

    pub fn root(&self) -> Hash {
        self.inner.hash.set_if_empty(|| {
            let padding = if self.is_frontier() {
                Hash::default()
            } else {
                todo!("one hash")
            };

            let hash = match self.inner() {
                Inner::Leaf(Leaf { commitment }) => match commitment {
                    Insert::Keep(commitment) => Hash::of(*commitment),
                    Insert::Hash(hash) => *hash,
                },
                Inner::Node(Node { children }) => {
                    let children_hashes: Vec<Hash> =
                        children.iter().map(|child| child.hash()).collect();

                    match children_hashes.as_slice() {
                        [] => unreachable!("nodes never have zero children"),
                        [a] => Hash::node(self.height(), *a, padding, padding, padding),
                        [a, b] => Hash::node(self.height(), *a, *b, padding, padding),
                        [a, b, c] => Hash::node(self.height(), *a, *b, *c, padding),
                        [a, b, c, d] => Hash::node(self.height(), *a, *b, *c, *d),
                        _ => unreachable!("nodes never have more than 4 children"),
                    }
                }
                Inner::Tier(Tier { root }) => match root {
                    None => padding,
                    Some(Insert::Keep(root)) => root.hash(),
                    Some(Insert::Hash(hash)) => *hash,
                },
            };

            hash
        })
    }
}

// This impl allows us to take the hash of `Insert<Tree>`, which is useful.
impl GetHash for Tree {
    fn hash(&self) -> Hash {
        self.root()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.inner.hash.get()
    }
}
