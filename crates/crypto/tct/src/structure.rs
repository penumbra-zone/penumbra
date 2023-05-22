//! A dynamic representation of nodes within the internal tree structure.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use crate::prelude::*;

#[doc(inline)]
pub use crate::internal::hash::{Forgotten, Hash};

/// Every kind of node in the tree implements [`Node`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub(crate) trait Any<'tree>: GetHash + sealed::Sealed {
    /// The children of this node.
    fn children(&'tree self) -> Vec<HashOrNode<'tree>>;

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    fn kind(&self) -> Kind;

    /// The most recent time something underneath this node was forgotten.
    fn forgotten(&self) -> Forgotten;
}

impl GetHash for &dyn Any<'_> {
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }

    fn clear_cached_hash(&self) {
        (**self).clear_cached_hash()
    }
}

impl<'tree, T: Any<'tree>> Any<'tree> for &T {
    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn forgotten(&self) -> Forgotten {
        (**self).forgotten()
    }

    fn children(&'tree self) -> Vec<HashOrNode<'tree>> {
        (**self).children()
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A leaf node at the bottom of the tree.
    Leaf {
        /// The witnessed commitment at this leaf, or `None` if this leaf was forgotten.
        commitment: Option<Commitment>,
    },
    /// An internal node within the tree.
    Internal {
        /// The height of this internal node.
        height: u8,
    },
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Leaf { .. } => write!(f, "Leaf",),
            Kind::Internal { .. } => write!(f, "Node"),
        }
    }
}

/// The place a node is located in a tree: whether it is on the frontier or is completed.
///
/// This is redundant with the pair of (height, index) if the total size of the tree is known, but
/// it is useful to reveal it directly.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// The node is not on the frontier.
    Complete,
    /// The node is on the frontier.
    Frontier,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Frontier => write!(f, "frontier"),
            Place::Complete => write!(f, "complete"),
        }
    }
}

/// An arbitrary node somewhere within a tree.
#[derive(Clone, Copy)]
pub struct Node<'tree> {
    offset: u64,
    global_position: Option<Position>,
    this: HashOrNode<'tree>,
}

impl GetHash for Node<'_> {
    fn hash(&self) -> Hash {
        self.this.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.this.cached_hash()
    }

    fn clear_cached_hash(&self) {
        self.this.clear_cached_hash()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HashOrNode<'tree> {
    Hash(HashedNode),
    Node(&'tree dyn Any<'tree>),
}

impl GetHash for HashOrNode<'_> {
    fn hash(&self) -> Hash {
        match self {
            HashOrNode::Hash(hashed) => hashed.hash(),
            HashOrNode::Node(node) => node.hash(),
        }
    }

    fn cached_hash(&self) -> Option<Hash> {
        match self {
            HashOrNode::Hash(hashed) => Some(hashed.hash()),
            HashOrNode::Node(node) => node.cached_hash(),
        }
    }

    fn clear_cached_hash(&self) {
        if let HashOrNode::Node(node) = self {
            node.clear_cached_hash()
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HashedNode {
    pub hash: Hash,
    pub height: u8,
    pub forgotten: Forgotten,
}

impl GetHash for HashedNode {
    fn hash(&self) -> Hash {
        self.hash
    }

    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }

    fn clear_cached_hash(&self) {}
}

impl Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{}::{}", self.place(), self.kind());
        let mut s = f.debug_struct(&name);
        if self.height() != 0 {
            s.field("height", &(*self).height());
        }
        s.field("position", &u64::from(self.position()));
        if self.forgotten() != Forgotten::default() {
            s.field("forgotten", &self.forgotten());
        }
        if let Some(hash) = self.cached_hash() {
            s.field("hash", &hash);
        }
        if let Kind::Leaf {
            commitment: Some(commitment),
        } = self.kind()
        {
            s.field("commitment", &commitment);
        }
        let children = self.children();
        if !children.is_empty() {
            s.field("children", &children);
        }
        s.finish()
    }
}

impl Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("position", &self.position())
            .finish_non_exhaustive()
    }
}

impl<'tree> Node<'tree> {
    /// Make a root node.
    pub(crate) fn root<R: Any<'tree> + GetPosition>(node: &'tree R) -> Self {
        Self {
            offset: 0,
            global_position: node.position().map(Into::into),
            this: HashOrNode::Node(node),
        }
    }

    /// The hash of this node.
    pub fn hash(&self) -> Hash {
        self.this.hash()
    }

    /// The cached hash at this node, if any.
    pub fn cached_hash(&self) -> Option<Hash> {
        self.this.cached_hash()
    }

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    pub fn kind(&self) -> Kind {
        match self.this {
            HashOrNode::Hash(HashedNode { height, .. }) => Kind::Internal { height },
            HashOrNode::Node(node) => node.kind(),
        }
    }

    /// The most recent time something underneath this node was forgotten.
    pub fn forgotten(&self) -> Forgotten {
        match self.this {
            HashOrNode::Hash(HashedNode { forgotten, .. }) => forgotten,
            HashOrNode::Node(node) => node.forgotten(),
        }
    }

    /// The children of this node.
    pub fn children(&self) -> Vec<Node<'tree>> {
        match self.this {
            HashOrNode::Hash(_) => Vec::new(),
            HashOrNode::Node(node) => node
                .children()
                .into_iter()
                .enumerate()
                .map(|(i, hash_or_node)| Node {
                    global_position: self.global_position,
                    offset: self.offset * 4 + (i as u64),
                    this: hash_or_node,
                })
                .collect(),
        }
    }

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    pub fn index(&self) -> u64 {
        self.offset
    }

    /// The height of this node above the base of the tree.
    pub fn height(&self) -> u8 {
        match self.kind() {
            Kind::Internal { height } => height,
            Kind::Leaf { .. } => 0,
        }
    }

    /// The position of the node (the vertical extension of the position of commitments below).
    pub fn position(&self) -> Position {
        (4u64.pow(self.height() as u32) * self.index()).into()
    }

    /// The distance between positions of nodes at this height.
    pub fn stride(&self) -> u64 {
        4u64.pow(self.height() as u32)
    }

    /// The range of positions that could occur beneath this node (not all of them need be actually
    /// represented in the tree).
    pub fn range(&self) -> Range<Position> {
        let position: u64 = self.position().into();
        position.into()..(position + self.stride()).min(4u64.pow(24) - 1).into()
    }

    /// The global position of the tree inside of which this node exists.
    pub fn global_position(&self) -> Option<Position> {
        self.global_position
    }

    /// The place on the tree where this node occurs.
    pub fn place(&self) -> Place {
        if let Some(global_position) = self.global_position() {
            if let Some(frontier_tip) = u64::from(global_position).checked_sub(1) {
                let height = self.height();
                let position = u64::from(self.position());
                if position >> (height * 2) == frontier_tip >> (height * 2) {
                    // The prefix of the position down to this height matches the path to the
                    // frontier tip, which means that this node is on the frontier
                    Place::Frontier
                } else {
                    // The prefix doesn't match (the node's position is less than the frontier, even
                    // considering only the prefix), so it's complete
                    Place::Complete
                }
            } else {
                // The global position is zero (the tree is empty, i.e. all nodes, that is, none,
                // are frontier nodes, vacuously)
                Place::Frontier
            }
        } else {
            // There is no global position (the tree is full, i.e. all nodes are complete)
            Place::Complete
        }
    }
}

mod sealed {
    use super::*;

    pub trait Sealed: Send + Sync {}

    impl<T: Sealed> Sealed for &T {}
    impl Sealed for Node<'_> {}

    impl Sealed for complete::Item {}
    impl<T: Sealed> Sealed for complete::Leaf<T> {}
    impl<T: Sealed + Clone> Sealed for complete::Node<T> {}
    impl<T: Sealed + Height + GetHash + Clone> Sealed for complete::Tier<T> {}
    impl<T: Sealed + Height + GetHash + Clone> Sealed for complete::Top<T> {}

    impl Sealed for frontier::Item {}
    impl<T: Sealed> Sealed for frontier::Leaf<T> {}
    impl<T: Sealed + Focus> Sealed for frontier::Node<T> where T::Complete: Send + Sync {}
    impl<T: Sealed + Height + GetHash + Focus + Clone> Sealed for frontier::Tier<T> where
        T::Complete: Send + Sync + Clone
    {
    }
    impl<T: Sealed + Height + GetHash + Focus + Clone> Sealed for frontier::Top<T> where
        T::Complete: Send + Sync + Clone
    {
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indexing_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        for i in 0..MAX_SIZE_TO_TEST {
            top.insert(Commitment(i.into()).into()).unwrap();
        }

        fn check_leaves(index: &mut [u64; 9], node: Node) {
            assert_eq!(node.index(), index[usize::from(node.height())], "{node}");

            index[usize::from(node.height())] += 1;

            for child in node.children() {
                check_leaves(index, child);
            }
        }

        check_leaves(&mut [0; 9], Node::root(&top));
    }

    #[test]
    fn place_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        for i in 0..MAX_SIZE_TO_TEST {
            top.insert(Commitment(i.into()).into()).unwrap();
            let root = Node::root(&top);
            check(root, Place::Frontier);
        }

        fn check(node: Node, expected: Place) {
            assert_eq!(node.place(), expected);
            match node.children().as_slice() {
                [] => {}
                [a] => {
                    check(*a, expected);
                }
                [a, b] => {
                    check(*a, Place::Complete);
                    check(*b, expected);
                }
                [a, b, c] => {
                    check(*a, Place::Complete);
                    check(*b, Place::Complete);
                    check(*c, expected);
                }
                [a, b, c, d] => {
                    check(*a, Place::Complete);
                    check(*b, Place::Complete);
                    check(*c, Place::Complete);
                    check(*d, expected);
                }
                _ => unreachable!("nodes can't have > 4 children"),
            }
        }
    }

    #[test]
    fn height_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut tree = crate::Tree::new();

        for i in 0..MAX_SIZE_TO_TEST {
            tree.insert(crate::Witness::Keep, Commitment(i.into()))
                .unwrap();
            let root = tree.structure();
            check(root, 24);
        }

        fn check(node: Node, expected: u8) {
            assert_eq!(node.height(), expected, "{node}");
            for child in node.children() {
                check(child, expected - 1);
            }
        }
    }
}
