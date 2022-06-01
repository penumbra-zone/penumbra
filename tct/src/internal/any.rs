//! A dynamic representation of nodes within the tree structure, for writing homogeneous traversals.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use crate::prelude::*;

/// Every kind of node in the tree implements [`Any`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub trait Any: GetHash {
    /// The kind of the node: either an internal node with a height, or a leaf with a commitment
    fn kind(&self) -> Kind;

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    fn index(&self) -> u64 {
        0
    }

    /// The position of the tree within which this node occurs.
    fn global_position(&self) -> Option<u64>;

    /// The children, or hashes of them, of this node.
    fn children(&self) -> Vec<(Insert<Child>, Forgotten)>;
}

impl<T: Any> Any for &T {
    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn index(&self) -> u64 {
        (**self).index()
    }

    fn global_position(&self) -> Option<u64> {
        (**self).global_position()
    }

    fn children(&self) -> Vec<(Insert<Child>, Forgotten)> {
        (**self).children()
    }
}

pub trait AnyExt: Any {
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

impl<T: Any + ?Sized> AnyExt for T {}

impl Debug for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Any")
            .field("height", &(*self).height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl Display for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Any")
            .field("height", &self.height())
            .field("index", &self.index())
            .finish_non_exhaustive()
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// The rightmost leaf of the tree.
    Rightmost(Option<Commitment>),
    /// A leaf node at the bottom of some tier.
    Leaf(Commitment),
    /// An internal node within some tier.
    Node(u8),
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Rightmost(option) => write!(
                f,
                "Rightmost({})",
                option
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "_".to_string())
            ),
            Kind::Leaf(commitment) => write!(f, "Leaf({})", commitment),
            Kind::Node(height) => write!(f, "Node({})", height),
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
pub struct Child<'a> {
    offset: u64,
    global_position: Option<u64>,
    inner: &'a dyn Any,
}

impl Debug for Child<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child")
            .field("height", &self.height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl<'a> Child<'a> {
    /// Make a new [`Child`] from a reference to something implementing [`Any`].
    pub fn new(parent: &'a dyn Any, child: &'a dyn Any) -> Self {
        Child {
            offset: 0,
            global_position: parent.global_position(),
            inner: child,
        }
    }
}

impl GetHash for Child<'_> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl Any for Child<'_> {
    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn index(&self) -> u64 {
        self.offset + self.inner.index()
    }

    fn global_position(&self) -> Option<u64> {
        self.global_position
    }

    fn children(&self) -> Vec<(Insert<Child>, Forgotten)> {
        self.inner
            .children()
            .into_iter()
            .enumerate()
            .map(|(nth, (child, forgotten))| {
                (
                    child.map(|child| {
                        debug_assert_eq!(
                            child.offset, 0,
                            "explicitly constructed children should have zero offset"
                        );
                        // If the height doesn't change, we shouldn't be applying a multiplier to the
                        // parent offset:
                        let multiplier = 4u64.pow((self.height() - child.height()).into());
                        Child {
                            inner: child.inner,
                            global_position: self.global_position,
                            offset: self.offset * multiplier + nth as u64,
                        }
                    }),
                    forgotten,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indexing_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new();
        for i in 0..=MAX_SIZE_TO_TEST {
            top.insert(Commitment(i.into()).into()).unwrap();
        }

        fn check_leaves(index: &mut [[u64; 5]; 9], node: &dyn Any) {
            assert_eq!(
                node.index(),
                index[usize::from(node.height())][node.kind() as usize],
                "{}",
                node
            );

            index[usize::from(node.height())][node.kind() as usize] += 1;

            for child in node
                .children()
                .iter()
                .filter_map(|child| child.as_ref().keep())
            {
                check_leaves(index, child);
            }
        }

        check_leaves(&mut [[0; 5]; 9], &top);
    }
}
