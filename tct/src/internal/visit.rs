//! A generic mechanism for writing traversals of the inner tree structure, i.e. for serialization,
//! structure validation, etc.

use crate::prelude::*;

mod any;
mod offset;

pub mod traversal;
pub use any::{Any, Kind, Place};

/// A visitor for a node within a tree.
pub trait Visitor {
    /// The output of each function: this must be the same for every possible kind of node.
    type Output;

    /// Visit an item on the frontier, the right-most leaf of the tree.
    fn frontier_item(&mut self, index: u64, item: &frontier::Item) -> Self::Output;

    /// Visit a leaf of some tier on the frontier.
    fn frontier_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &frontier::Leaf<Child>,
    ) -> Self::Output;

    /// Visit an internal node along the frontier.
    fn frontier_node<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        node: &frontier::Node<Child>,
    ) -> Self::Output;

    /// Visit the root of a tier along the frontier.
    fn frontier_tier<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        tier: &frontier::Tier<Child>,
    ) -> Self::Output;

    /// Visit the top of a frontier tree.
    fn frontier_top<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        top: &frontier::Top<Child>,
    ) -> Self::Output;

    /// Visit a complete item at the bottom edge of the tree somewhere.
    fn complete_item(&mut self, index: u64, item: &complete::Item) -> Self::Output;

    /// Visit a leaf of some tier within the tree.
    fn complete_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &complete::Leaf<Child>,
    ) -> Self::Output;

    /// Visit an internal node within the tree that is not on the frontier.
    fn complete_node<Child: Height + GetHash>(
        &mut self,
        index: u64,
        node: &complete::Node<Child>,
    ) -> Self::Output;

    /// Visit the root of a complete tier within the tree.
    fn complete_tier<Child: Height + GetHash>(
        &mut self,
        index: u64,
        tier: &complete::Tier<Child>,
    ) -> Self::Output;

    /// Visit the top of a completed tree.
    fn complete_top<Child: Height + GetHash>(
        &mut self,
        index: u64,
        top: &complete::Top<Child>,
    ) -> Self::Output;
}

/// An abstraction of tree traversal order, so the contents of the traversal can be separated from
/// the order in which it is performed.
pub trait Traversal {
    /// Traverse a node somewhere in the tree.
    ///
    /// By default, this defines the behavior for both frontier and complete nodes; override
    /// [`Traversal::traverse_complete`] to change this.
    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    );

    /// Traverse a node somewhere in the complete part of the tree.
    ///
    /// This does not need to be overridden unless you want different traversal behavior in the
    /// complete part of the tree.
    fn traverse_complete<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
    ) {
        self.traverse(
            visitor,
            output,
            parent,
            complete_children,
            None::<std::convert::Infallible>,
        );
    }
}

/// Nodes within the tree implement this trait to allow themselves to be visited.
pub trait Visit: GetHash {
    /// Visit this node with the provided visitor.
    ///
    /// There is never a need to override this; define `visit_indexed` instead.
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        self.visit_indexed(0, visitor)
    }

    /// Visit this node using the provided visitor and index.
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output;
}

/// Nodes within the tree implement this trait to allow themselves to be traversed.
pub trait Traverse {
    /// Traverse this node using the provided traversal, visitor, and output function.
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    );
}

/// Offset the traversal of a child by the appropriate amount, given the child's height and index
/// within its parent.
pub(crate) fn child<Child: Height + Traverse>(
    n: usize,
    child: Option<&Child>,
) -> impl Traverse + '_ {
    offset::Offset {
        inner: child,
        offset: (n as u64) * 4u64.pow(Child::Height::HEIGHT.into()),
    }
}

// It's useful to traverse `Option`s:

impl<S: Traverse> Traverse for Option<S> {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        if let Some(v) = self.as_ref() {
            v.traverse(traversal, visitor, output)
        }
    }
}

// References can be visited and traversed also:

impl<T: Visit> Visit for &T {
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output {
        (*self).visit_indexed(index, visitor)
    }
}

impl<S: Traverse> Traverse for &S {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        (*self).traverse(traversal, visitor, output);
    }
}

// We need to implement `Visit` and `Traverse` for `std::convert::Infallible` because we need to
// define the child type for `Item`, even though it never has any children:

pub(crate) const NO_CHILDREN: [std::convert::Infallible; 0] = [];
pub(crate) const NO_CHILD: Option<std::convert::Infallible> = None;

// Necessary because we want to make Infallible a vacuous Visit:
impl GetHash for std::convert::Infallible {
    fn hash(&self) -> Hash {
        unreachable!("std::convert::Infallible can't be constructed")
    }

    fn cached_hash(&self) -> Option<Hash> {
        unreachable!("std::convert::Infallible can't be constructed")
    }
}

impl Visit for std::convert::Infallible {
    #[inline]
    fn visit_indexed<V: Visitor>(&self, _index: u64, _visitor: &mut V) -> V::Output {
        // never do anything, because there's nothing here to visit
        unreachable!("can't construct an infallible item, so it can't be visited")
    }
}

impl Traverse for std::convert::Infallible {
    #[inline]
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        _traversal: &mut T,
        _visitor: &mut V,
        _output: &mut impl FnMut(V::Output),
    ) {
        // never do anything, because there's nothing here to traverse
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_leaves_correct() {
        let mut block: frontier::Top<Item> = frontier::Top::new();
        for n in 0u64..10 {
            block.insert(Commitment(n.into()).into()).unwrap();
        }

        let mut nodes = Vec::new();
        block.traverse(
            &mut traversal::PostOrder,
            &mut |n| nodes.push(n),
            &mut |_| (),
        );

        for item in nodes {
            println!("{item:?}");
        }

        panic!();
    }
}
