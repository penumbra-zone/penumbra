//! The definitions of the core tree structure.

use std::sync::{Arc, Weak};

use crate::*;

mod build;
use build::Builder;

/// A tiered commitment tree.
pub struct Tree {
    inner: Arc<Inner>,
}

/// A weak reference to the parent of a node, defined as a convenience synonym.
type Parent = Weak<Inner>;

/// The interior of a tree node.
struct Inner {
    /// The parent pointer for this node.
    parent: Parent,
    /// The height of this node above the level of the leaves of the tree.
    height: u8,
    /// The actual node itself, which can be a [`Tier`], [`Internal`] node, or [`Leaf`].
    node: Node,
    /// The hash of this node, cached so that it is only computed once.
    hash: CachedHash,
}

impl Inner {
    /// Construct a new `Inner`.
    pub fn new(parent: Parent, height: u8, node: Node) -> Self {
        Self {
            parent,
            height,
            node,
            hash: CachedHash::default(),
        }
    }
}

/// A node in the tree.
enum Node {
    /// A leaf node.
    Leaf(Leaf),
    /// An internal node.
    Internal(Internal),
    /// A tier, wrapping a subtree.
    Tier(Tier),
}

/// A leaf node.
pub struct Leaf {
    /// The contained commitment in this leaf, or if it was not witnessed, its hash.
    pub commitment: Insert<Commitment>,
}

/// An internal node.
pub struct Internal {
    /// The children of this node.
    ///
    /// Invariant: the length of this vector is between 1 and 4.
    pub children: Vec<Tree>,
}

/// A tier node.
pub struct Tier {
    /// The wrapped subtree, which can be `None` if the tier is empty, a hash if it was summarized
    /// by a hash, or a non-empty subtree otherwise.
    pub root: Option<Insert<Tree>>,
}

impl Tree {
    /// Check whether this tree is referentially equal to another, by interior pointer equality.
    fn is(&self, other: &Tree) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Get the parent, if there is one, of this tree.
    fn parent(&self) -> Option<Tree> {
        self.inner.parent.upgrade().map(|inner| Tree { inner })
    }

    /// Get the wrapped [`Node`] at this level of the tree.
    fn inner(&self) -> &Node {
        &self.inner.node
    }

    /// Construct a one-tiered tree from an iterable sequence representing commitments in the block.
    pub fn from_block(block: List<Insert<Commitment>>) -> Tree {
        build::block(Insert::Keep(block)).with_parent(Parent::new())
    }

    /// Construct a two-tiered tree from a doubly-nested iterable sequence representing commitments
    /// within blocks.
    pub fn from_epoch(epoch: List<Insert<List<Insert<Commitment>>>>) -> Tree {
        build::epoch(Insert::Keep(epoch)).with_parent(Parent::new())
    }

    /// Construct a three-tiered tree from a triply-nested iterable sequence representing commitments within
    /// blocks within epochs.
    pub fn from_eternity(eternity: List<Insert<List<Insert<List<Insert<Commitment>>>>>>) -> Tree {
        build::eternity(eternity).with_parent(Parent::new())
    }

    /// Assuming this tree is a [`Leaf`], cast it as one.
    ///
    /// # Panics
    ///
    /// Panics if this tree is not a [`Leaf`].
    pub fn as_leaf(&self) -> &Leaf {
        if let Node::Leaf(leaf) = self.inner() {
            leaf
        } else {
            panic!("expected tree to be a leaf")
        }
    }

    /// Assuming this tree is an [`Internal`] node, cast it as one.
    ///
    /// # Panics
    ///
    /// Panics if this tree is not an [`Internal`] node.
    pub fn as_internal(&self) -> &Internal {
        if let Node::Internal(node) = self.inner() {
            node
        } else {
            panic!("expected tree to be a node")
        }
    }

    /// Assuming this tree is a [`Tier`], cast it as one.
    ///
    /// # Panics
    ///
    /// Panics if this tree is not a [`Tier`].
    pub fn as_tier(&self) -> &Tier {
        if let Node::Tier(tier) = self.inner() {
            tier
        } else {
            panic!("expected tree to be a tier")
        }
    }

    /// Determine whether this (sub)tree is on the _frontier_: whether it is on the rightmost edge
    /// of its containing tree.
    fn is_frontier(&self) -> bool {
        if let Some(parent) = self.parent() {
            match parent.inner() {
                Node::Tier(_) => parent.is_frontier(),
                Node::Internal(node) => {
                    node.children.last().unwrap().is(self) && parent.is_frontier()
                }
                Node::Leaf(_) => unreachable!("the parent of a tree can never be a leaf"),
            }
        } else {
            true
        }
    }

    /// Get the height of this tree.
    fn height(&self) -> u8 {
        self.inner.height
    }

    /// Compute the root hash of this tree.
    pub fn root(&self) -> Hash {
        self.inner.hash.set_if_empty(|| {
            let hash = match self.inner() {
                Node::Leaf(Leaf { commitment }) => match commitment {
                    Insert::Keep(commitment) => Hash::of(*commitment),
                    Insert::Hash(hash) => *hash,
                },
                Node::Internal(_) => {
                    let [a, b, c, d] = self.padded_children().unwrap().map(|child| child.hash());
                    Hash::node(self.height(), a, b, c, d)
                }
                Node::Tier(Tier { root }) => match root {
                    None => self.padding_hash(),
                    Some(Insert::Keep(root)) => root.hash(),
                    Some(Insert::Hash(hash)) => *hash,
                },
            };

            hash
        })
    }

    /// Get the correct padding hash for this node in the tree, depending on whether it is on the
    /// frontier.
    fn padding_hash(&self) -> Hash {
        if self.is_frontier() {
            Hash::zero()
        } else {
            Hash::one()
        }
    }

    /// Get the four children of this tree, padded with padding hashes if necessary.
    ///
    /// Returns `None` if this tree is a leaf, an empty tier, or a hashed tier.
    pub fn padded_children(&self) -> Option<[Insert<&Tree>; 4]> {
        use Insert::*;

        let padding = Hash(self.padding_hash());

        match self.inner() {
            Node::Leaf(_) => None,
            Node::Internal(node) => Some(match node.children.as_slice() {
                [] => unreachable!("nodes never have zero children"),
                [a] => [Keep(a), padding, padding, padding],
                [a, b] => [Keep(a), Keep(b), padding, padding],
                [a, b, c] => [Keep(a), Keep(b), Keep(c), padding],
                [a, b, c, d] => [Keep(a), Keep(b), Keep(c), Keep(d)],
                _ => unreachable!("nodes never have more than 4 children"),
            }),
            Node::Tier(tier) => match tier.root {
                None | Some(Hash(_)) => None,
                Some(Keep(ref root)) => root.padded_children(),
            },
        }
    }

    /// Get the authentication path for a given position, assuming that the tree is of the height
    /// equal to the length inferred for the returned array.
    ///
    /// # Panics
    ///
    /// If the index does not correspond to a leaf in the tree, or if the inferred authentication
    /// path length does not match the actual height of the tree.
    pub fn witness<const HEIGHT: usize>(&self, mut index: u64) -> [[Hash; 3]; HEIGHT] {
        assert_eq!(
            HEIGHT,
            self.height() as usize,
            "inferred authentication path length must match height of tree"
        );

        let mut path = Vec::with_capacity(HEIGHT);
        let mut tree = self;

        while tree.height() > 0 {
            // Determine which way down to go, and what the next index should be
            let (which_way, child_index) = WhichWay::at(tree.height(), index);

            // Get the children of this node
            let children = tree.padded_children().expect("index is not witnessed");

            // Pick the child to witness and its siblings
            if let (Insert::Keep(child), siblings) = which_way.pick(children) {
                // Compute the hash of each sibling and push them onto the auth path
                path.push(siblings.map(|sibling| sibling.hash()));

                // Proceed in the next iteration to witness the child
                tree = child;
                index = child_index;
            } else {
                panic!("index is not witnessed")
            }
        }

        path.try_into()
            .expect("authentication path is always of correct length")
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
