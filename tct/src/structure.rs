//! A dynamic representation of nodes within the internal tree structure.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use crate::prelude::*;

/// Every kind of node in the tree implements [`Any`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub trait Node: GetHash + sealed::Sealed {
    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    fn index(&self) -> u64 {
        0
    }

    /// The kind of the node: either an internal node with a height, or a leaf with a commitment
    fn kind(&self) -> Kind;

    /// The position of the tree within which this node occurs.
    fn global_position(&self) -> Option<u64>;

    /// The parent of this node, if any.
    fn parent(&self) -> Option<&dyn Node> {
        None
    }

    /// The most recent time something underneath this node was forgotten.
    fn forgotten(&self) -> Forgotten;

    /// The children, or hashes of them, of this node.
    fn children(&self) -> Vec<Child>;

    // All of these methods are implemented in terms of the ones above:

    /// The height of this node above the base of the tree.
    fn height(&self) -> u8 {
        match self.kind() {
            Kind::Node(height) => height,
            Kind::Leaf(_) => 0,
        }
    }

    /// The position of the node (the vertical extension of the position of commitments below).
    fn position(&self) -> u64 {
        4u64.pow(self.height() as u32) * self.index()
    }

    /// The range of positions that occur beneath this node.
    fn range(&self) -> Range<u64> {
        let position = self.position();
        position..position + 4u64.pow(self.height() as u32)
    }

    /// The place on the tree where this node occurs.
    fn place(&self) -> Place {
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

impl GetHash for &dyn Node {
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }
}

impl<T: Node> Node for &T {
    fn index(&self) -> u64 {
        (**self).index()
    }

    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn global_position(&self) -> Option<u64> {
        (**self).global_position()
    }

    fn parent(&self) -> Option<&dyn Node> {
        (**self).parent()
    }

    fn forgotten(&self) -> Forgotten {
        (**self).forgotten()
    }

    fn children(&self) -> Vec<Child> {
        (**self).children()
    }
}

impl Debug for &dyn Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &(*self).height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl Display for &dyn Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("index", &self.index())
            .finish_non_exhaustive()
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A leaf node at the bottom of some tier.
    Leaf(Option<Commitment>),
    /// An internal node within some tier.
    Node(u8),
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Leaf(_) => write!(f, "Leaf",),
            Kind::Node(_) => write!(f, "Node"),
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

/// A child of an [`Any`]: this implements [`Any`] and supertraits, so can and should be treated
/// equivalently.
#[derive(Copy, Clone)]
pub struct Child<'a> {
    offset: u64,
    forgotten: Forgotten,
    parent: &'a (dyn Node + 'a),
    child: Insert<&'a (dyn Node + 'a)>,
}

impl Debug for Child<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl<'a> Child<'a> {
    /// Make a new [`Child`] from a reference to something implementing [`Any`].
    pub fn new(parent: &'a dyn Node, forgotten: Forgotten, child: Insert<&'a dyn Node>) -> Self {
        Child {
            offset: 0,
            forgotten,
            parent,
            child,
        }
    }
}

impl GetHash for Child<'_> {
    fn hash(&self) -> Hash {
        self.child.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.child.cached_hash()
    }
}

impl Node for Child<'_> {
    fn index(&self) -> u64 {
        self.offset
    }

    fn parent(&self) -> Option<&dyn Node> {
        Some(self.parent)
    }

    fn kind(&self) -> Kind {
        match self.child {
            Insert::Keep(child) => child.kind(),
            Insert::Hash(_) => match self.parent.kind() {
                Kind::Node(height @ 2..=24) => Kind::Node(height - 1),
                Kind::Node(1) => Kind::Leaf(None),
                Kind::Node(0 | 25..=u8::MAX) => {
                    unreachable!("nodes cannot have zero height or height greater than 24")
                }
                Kind::Leaf(_) => unreachable!("leaves cannot have children"),
            },
        }
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten
    }

    fn global_position(&self) -> Option<u64> {
        self.parent.global_position()
    }

    fn children(&self) -> Vec<Child> {
        if let Insert::Keep(child) = self.child {
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
                    Child {
                        forgotten: child.forgotten,
                        child: child.child,
                        parent: child.parent,
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
    impl Sealed for Child<'_> {}

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

        fn check_leaves(index: &mut [u64; 9], node: &dyn Node) {
            assert_eq!(node.index(), index[usize::from(node.height())], "{}", node);

            index[usize::from(node.height())] += 1;

            for child in node.children() {
                check_leaves(index, &child);
            }
        }

        check_leaves(&mut [0; 9], &top);
    }
}
