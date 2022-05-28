//! A dynamic representation of nodes within the tree structure, for writing homogeneous traversals.

use std::fmt::{Debug, Display};

use crate::prelude::*;

/// Every kind of node in the tree implements [`Any`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub trait Any: GetHash {
    /// The place this node is located: on the frontier or in the complete interior.
    fn place(&self) -> Place;

    /// The kind of node this is: an item at the base, a leaf of some tier, an internal node, a
    /// tier root, or a top-level root.
    fn kind(&self) -> Kind;

    /// The height of this node above the base of the tree.
    fn height(&self) -> u8;

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    fn index(&self) -> u64 {
        0
    }

    /// The children, or hashes of them, of this node.
    fn children(&self) -> Vec<Insert<Child>>;
}

impl Debug for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Any")
            .field("place", &self.place())
            .field("kind", &self.kind())
            .field("height", &self.height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl Display for &dyn Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{} {{ height: {}, index: {} }}", self.place(), self.kind(), self.height(), self.index())
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// An item node at the bottom of the tree.
    Item,
    /// A leaf node at the bottom of some tier.
    Leaf,
    /// An internal node within some tier.
    Node,
    /// The root of a tier node.
    Tier,
    /// The top of a tree.
    Top,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Item => write!(f, "Item"),
            Kind::Leaf => write!(f, "Leaf"),
            Kind::Node => write!(f, "Node"),
            Kind::Tier => write!(f, "Tier"),
            Kind::Top => write!(f, "Top"),
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
    inner: &'a dyn Any,
}

impl Debug for Child<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Child")
            .field("place", &self.place())
            .field("kind", &self.kind())
            .field("height", &self.height())
            .field("index", &self.index())
            .field("children", &self.children())
            .finish()
    }
}

impl<'a> Child<'a> {
    /// Make a new [`Child`] from a reference to something implementing [`Any`].
    pub fn new(child: &'a dyn Any) -> Self {
        Child {
            offset: 0,
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
    fn place(&self) -> Place {
        self.inner.place()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }

    fn height(&self) -> u8 {
        self.inner.height()
    }

    fn index(&self) -> u64 {
        self.offset + self.inner.index()
    }

    fn children(&self) -> Vec<Insert<Child>> {
        self.inner
            .children()
            .into_iter()
            .enumerate()
            .map(|(nth, child)| {
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
                        offset: self.offset * multiplier + nth as u64,
                    }
                })
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
            assert_eq!(node.index(), index[usize::from(node.height())][node.kind() as usize], "{}", node);

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
