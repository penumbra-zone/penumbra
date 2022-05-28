//! A dynamic representation of nodes within the tree structure, for writing homogeneous traversals.

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Complete,
    Frontier,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Item,
    Leaf,
    Node,
    Tier,
    Top,
}

/// A child of an [`Any`]: this implements [`Any`] and supertraits, so can and should be treated
/// equivalently.
pub struct Child<'a> {
    offset: u64,
    inner: &'a dyn Any,
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
                child.map(|child| Child {
                    inner: child.inner,
                    offset: self.offset * 4 + child.offset + nth as u64,
                })
            })
            .collect()
    }
}
