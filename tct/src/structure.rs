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
pub(crate) trait Any: GetHash + sealed::Sealed {
    /// The parent of this node, if any.
    ///
    /// This defaults to `None`, but is filled in by the [`Any`] implementation of [`Node`].
    fn parent(&self) -> Option<Node> {
        None
    }

    /// The children of this node.
    fn children(&self) -> Vec<Node>;

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    fn kind(&self) -> Kind;

    /// The most recent time something underneath this node was forgotten.
    fn forgotten(&self) -> Forgotten;

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    ///
    /// This defaults to 0, but is filled in by the [`Any`] implementation of [`Node`].
    fn index(&self) -> u64 {
        0
    }

    /// The position of the tree within which this node occurs.
    fn global_position(&self) -> Option<Position>;
}

impl GetHash for &dyn Any {
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }
}

impl<T: Any> Any for &T {
    fn index(&self) -> u64 {
        (**self).index()
    }

    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn global_position(&self) -> Option<Position> {
        (**self).global_position()
    }

    fn parent(&self) -> Option<Node> {
        (**self).parent()
    }

    fn forgotten(&self) -> Forgotten {
        (**self).forgotten()
    }

    fn children(&self) -> Vec<Node> {
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
#[derive(Copy, Clone)]
pub struct Node<'a> {
    offset: u64,
    forgotten: Forgotten,
    parent: Option<&'a Node<'a>>,
    this: Insert<&'a (dyn Any + 'a)>,
}

impl Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &(*self).height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("index", &self.index())
            .finish_non_exhaustive()
    }
}

impl<'a> Node<'a> {
    /// Make a root node.
    pub(crate) fn root(this: &'a dyn Any) -> Self {
        Self {
            offset: 0,
            forgotten: this.forgotten(),
            parent: None,
            this: Insert::Keep(this),
        }
    }

    /// Make a new [`Child`] from a reference to something implementing [`Node`].
    pub(crate) fn child(forgotten: Forgotten, child: Insert<&'a dyn Any>) -> Self {
        Node {
            offset: 0,
            forgotten,
            parent: None,
            this: child,
        }
    }

    /// The parent of this node, if any.
    pub fn parent(&self) -> Option<Node> {
        (self as &dyn Any).parent()
    }

    /// The children of this node.
    pub fn children(&self) -> Vec<Node> {
        (self as &dyn Any).children()
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
        (self as &dyn Any).kind()
    }

    /// The most recent time something underneath this node was forgotten.
    pub fn forgotten(&self) -> Forgotten {
        (self as &dyn Any).forgotten()
    }

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    pub fn index(&self) -> u64 {
        (self as &dyn Any).index()
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

    /// The range of positions that could occur beneath this node (not all of them need be actually
    /// represented in the tree).
    pub fn range(&self) -> Range<Position> {
        let position: u64 = self.position().into();
        position.into()..(position + 4u64.pow(self.height() as u32)).into()
    }

    /// The place on the tree where this node occurs.
    pub fn place(&self) -> Place {
        if let Some(global_position) = self.global_position() {
            if self.range().end >= global_position {
                Place::Frontier
            } else {
                Place::Complete
            }
        } else {
            Place::Complete
        }
    }
}

impl GetHash for Node<'_> {
    fn hash(&self) -> Hash {
        self.this.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.this.cached_hash()
    }
}

impl Any for Node<'_> {
    fn index(&self) -> u64 {
        self.offset
    }

    fn parent(&self) -> Option<Node> {
        self.parent.copied()
    }

    fn kind(&self) -> Kind {
        match self.this {
            Insert::Keep(child) => child.kind(),
            Insert::Hash(_) => {
                if let Some(parent) = self.parent {
                    match parent.kind() {
                        Kind::Internal {
                            height: height @ 2..=24,
                        } => Kind::Internal { height: height - 1 },
                        Kind::Internal { height: 1 } => Kind::Leaf { commitment: None },
                        Kind::Internal {
                            height: 0 | 25..=u8::MAX,
                        } => {
                            unreachable!("nodes cannot have zero height or height greater than 24")
                        }
                        Kind::Leaf { .. } => unreachable!("leaves cannot have children"),
                    }
                } else {
                    // Hashed root node is an internal node of height 24
                    Kind::Internal { height: 24 }
                }
            }
        }
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten
    }

    fn global_position(&self) -> Option<Position> {
        self.parent.and_then(Any::global_position)
    }

    fn children(&self) -> Vec<Node> {
        if let Insert::Keep(child) = self.this {
            child
                .children()
                .into_iter()
                .enumerate()
                .map(|(nth, child)| {
                    debug_assert_eq!(
                        child.offset, 0,
                        "explicitly constructed children should have zero offset"
                    );
                    // If the height doesn't change, we shouldn't be applying a multiplier to the
                    // parent offset:
                    let multiplier = 4u64.pow((self.height() - child.height()).into());
                    Node {
                        forgotten: child.forgotten,
                        this: child.this,
                        parent: Some(self),
                        offset: self.offset * multiplier + nth as u64,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl<T: Sealed> Sealed for &T {}
    impl Sealed for Node<'_> {}

    impl Sealed for complete::Item {}
    impl<T: Sealed> Sealed for complete::Leaf<T> {}
    impl<T: Sealed> Sealed for complete::Node<T> {}
    impl<T: Sealed + Height + GetHash> Sealed for complete::Tier<T> {}
    impl<T: Sealed + Height + GetHash> Sealed for complete::Top<T> {}

    impl Sealed for frontier::Item {}
    impl<T: Sealed> Sealed for frontier::Leaf<T> {}
    impl<T: Sealed + Focus> Sealed for frontier::Node<T> {}
    impl<T: Sealed + Height + GetHash + Focus> Sealed for frontier::Tier<T> {}
    impl<T: Sealed + Height + GetHash + Focus> Sealed for frontier::Top<T> {}
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
            assert_eq!(node.index(), index[usize::from(node.height())], "{}", node);

            index[usize::from(node.height())] += 1;

            for child in node.children() {
                check_leaves(index, child);
            }
        }

        check_leaves(&mut [0; 9], Node::root(&top));
    }

    #[test]
    fn parent_of_child() {
        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        top.insert(Commitment(0u8.into()).into()).unwrap();

        // This can't be a loop, it has to be recursive because the lifetime parameter of the parent
        // is different for each recursive call
        fn check(parent: Node) {
            if let Some(child) = parent.children().pop() {
                let parent_of_child = child.parent().expect("child has no parent");
                assert_eq!(
                    parent.hash(),
                    parent_of_child.hash(),
                    "parent hash mismatch"
                );
                check(child);
            } else {
                assert_eq!(parent.height(), 0, "got all the way to a leaf");
            }
        }

        let root = Node::root(&top);
        check(root);
    }
}
