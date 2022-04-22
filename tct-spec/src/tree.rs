use std::{
    collections::VecDeque as List,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

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

    fn parent(&self) -> Option<Tree> {
        self.inner.parent.upgrade().map(|inner| Tree { inner })
    }

    pub fn from_block(block: impl IntoIterator<Item = Insert<Commitment>>) -> Tree {
        let mut block = build::block(block).with_parent(Parent::new());
        block.prune();
        block
    }

    pub fn from_epoch(
        epoch: impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
    ) -> Tree {
        let mut epoch = build::epoch(epoch).with_parent(Parent::new());
        epoch.prune();
        epoch
    }

    pub fn from_eternity(
        eternity: impl IntoIterator<
            Item = Insert<
                impl IntoIterator<Item = Insert<impl IntoIterator<Item = Insert<Commitment>>>>,
            >,
        >,
    ) -> Tree {
        let mut eternity = build::eternity(eternity).with_parent(Parent::new());
        eternity.prune();
        eternity
    }

    fn inner(&self) -> &Inner {
        &self.inner.inner
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

    pub fn root(&self) -> Hash {
        self.inner.hash.set_if_empty(|| {
            let padding = if self.is_frontier() {
                Hash::default()
            } else {
                todo!("one hash")
            };

            let hash = match self.inner() {
                Inner::Leaf(Leaf { commitment }) => Hash::of(*commitment),
                Inner::Node(Node { children }) => {
                    let children_hashes: Vec<Hash> =
                        children.iter().map(|child| child.lock().hash()).collect();

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

    fn prune(&mut self) -> bool {
        fn prune_insert(insert: &mut Insert<Tree>) -> bool {
            match insert {
                Insert::Keep(tree) => {
                    // Determine whether to keep the tree, in the process pruning everything beneath
                    // its topmost level
                    let keep = tree.prune();
                    if !keep {
                        // If the tree should not be kept, replace it with its hash
                        *insert = Insert::Hash(tree.hash());
                    }
                    keep
                }
                // A hashed tree should not be kept
                Insert::Hash(_) => false,
            }
        }

        match self.inner() {
            // A leaf containing a commitment is witnessed, so should be kept:
            Inner::Leaf(_) => true,
            // A node should be kept if any of its children are kept:
            Inner::Node(node) => node
                .children
                .iter()
                .any(|child| prune_insert(&mut child.lock())),
            // A tier should be kept if it is not empty and its root is kept:
            Inner::Tier(tier) => match tier.root {
                None => false,
                Some(ref root) => prune_insert(&mut root.lock()),
            },
        }
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
