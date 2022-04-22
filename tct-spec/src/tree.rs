use std::{
    collections::VecDeque as List,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

use penumbra_tct::{
    internal::{
        active::Insert,
        hash::{CachedHash, Hash},
    },
    Commitment,
};

type Parent = Weak<Wrapped>;

pub struct Tree {
    inner: Arc<Wrapped>,
}

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

pub enum Inner {
    Leaf(Leaf),
    Node(Node),
    Tier(Tier),
}

pub struct Leaf {
    pub commitment: Commitment,
}

pub struct Node {
    pub children: Vec<Mutex<Insert<Tree>>>,
}

pub struct Tier {
    pub root: Option<Mutex<Insert<Tree>>>,
}

mod build;
use build::Builder;

impl Tree {
    fn is(&self, other: &Tree) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn from_block(block: impl IntoIterator<Item = Insert<Commitment>>) -> Tree {
        build::block(block).with_parent(Parent::new())
    }

    pub fn from_epoch(
        epoch: impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
    ) -> Tree {
        build::epoch(epoch).with_parent(Parent::new())
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

    pub fn inner(&self) -> &Inner {
        &self.inner.inner
    }

    pub fn parent(&self) -> Option<Tree> {
        self.inner.parent.upgrade().map(|inner| Tree { inner })
    }

    pub fn is_frontier(&self) -> bool {
        if let Some(parent) = self.parent() {
            match parent.inner() {
                Inner::Tier(_) => parent.is_frontier(),
                Inner::Node(node) => match &*node.children.last().unwrap().lock() {
                    Insert::Keep(last_child) => last_child.is(self) && parent.is_frontier(),
                    Insert::Hash(_) => false,
                },
                Inner::Leaf(_) => unreachable!("the parent of a tree can never be a leaf"),
            }
        } else {
            true
        }
    }

    pub fn height(&self) -> u8 {
        self.inner.height
    }

    pub fn hash(&self) -> Hash {
        self.inner.hash.set_if_empty(|| {
            let padding = if self.is_frontier() {
                Hash::default()
            } else {
                todo!("one hash")
            };

            let hash = match self.inner() {
                Inner::Leaf(Leaf { commitment }) => Hash::of(*commitment),
                Inner::Node(Node { children }) => {
                    let children_hashes: Vec<Hash> = children
                        .iter()
                        .map(|child| match &*child.lock() {
                            Insert::Keep(child) => child.hash(),
                            Insert::Hash(hash) => *hash,
                        })
                        .collect();

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
                    Some(root) => match &*root.lock() {
                        Insert::Keep(root) => root.hash(),
                        Insert::Hash(hash) => *hash,
                    },
                },
            };

            hash
        })
    }

    pub fn prune(&self) -> bool {
        todo!("implement pruning")
    }
}
