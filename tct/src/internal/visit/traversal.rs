//! Common traversal patterns.

use super::*;

/// A post-order traversal that visits each child in left-to-right order before visiting the parent
/// of those children.
pub struct PostOrder<F, G> {
    /// Skip the parent if this function returns false.
    pub parent_filter: F,
    /// Skip the children if this function returns false.
    pub child_filter: G,
}

impl<F, G> Traversal for PostOrder<F, G>
where
    F: FnMut(&Any) -> bool,
    G: FnMut(&Any) -> bool,
{
    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    ) {
        let parent_any = Any::from(&parent);

        if (self.child_filter)(&parent_any) {
            for child in complete_children {
                child.traverse(self, visitor, output);
            }
            frontier_child.traverse(self, visitor, output);
        }

        if !(self.parent_filter)(&parent_any) {
            return;
        }

        parent.visit(visitor);
    }
}

/// A pre-order traversal that visits each child in left-to-right order after visiting the
/// parent of those children.
pub struct PreOrder<F, G> {
    /// Skip the parent if this function returns false.
    pub parent_filter: F,
    /// Skip the children if this function returns false.
    pub child_filter: G,
}

impl<F, G> Traversal for PreOrder<F, G>
where
    F: FnMut(&Any) -> bool,
    G: FnMut(&Any) -> bool,
{
    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    ) {
        let parent_any = Any::from(&parent);

        if (self.parent_filter)(&parent_any) {
            parent.visit(visitor);
        }

        if !(self.child_filter)(&parent_any) {
            return;
        }

        for child in complete_children {
            child.traverse(self, visitor, output);
        }
        frontier_child.traverse(self, visitor, output);
    }
}
