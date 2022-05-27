//! Common traversal patterns.

use super::*;

/// A post-order traversal that visits each child in left-to-right order before visiting the
/// parent of those children.
pub struct PostOrder;

impl Traversal for PostOrder {
    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    ) {
        for child in complete_children {
            child.traverse(self, visitor, output);
        }
        frontier_child.traverse(self, visitor, output);
        parent.visit(visitor);
    }
}

/// A pre-order traversal that visits each child in left-to-right order after visiting the
/// parent of those children.
pub struct PreOrder;

impl Traversal for PreOrder {
    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    ) {
        parent.visit(visitor);
        for child in complete_children {
            child.traverse(self, visitor, output);
        }
        frontier_child.traverse(self, visitor, output);
    }
}
