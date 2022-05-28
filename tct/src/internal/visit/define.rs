use super::*;

/// A definition of a visitor for a node within a tree, where some methods may be defaulted for
/// succinctness.
///
/// All `T: DefineVisitor` implement `Visitor` automatically.
pub trait DefineVisitor {
    /// The output of each function: this must be the same for every possible kind of node.
    type Output: Default;

    /// Visit an item on the frontier, the right-most leaf of the tree.
    fn frontier_item(&mut self, _index: u64, _item: &frontier::Item) -> Self::Output {
        Self::Output::default()
    }

    /// Visit a leaf of some tier on the frontier.
    fn frontier_leaf<Child: Height + GetHash>(
        &mut self,
        _index: u64,
        _leaf: &frontier::Leaf<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit an internal node along the frontier.
    fn frontier_node<Child: Height + Focus + GetHash>(
        &mut self,
        _index: u64,
        _node: &frontier::Node<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit the root of a tier along the frontier.
    fn frontier_tier<Child: Height + Focus + GetHash>(
        &mut self,
        _index: u64,
        _tier: &frontier::Tier<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit the top of a frontier tree.
    fn frontier_top<Child: Height + Focus + GetHash>(
        &mut self,
        _index: u64,
        _top: &frontier::Top<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit a complete item at the bottom edge of the tree somewhere.
    fn complete_item(&mut self, _index: u64, _item: &complete::Item) -> Self::Output {
        Self::Output::default()
    }

    /// Visit a leaf of some tier within the tree.
    fn complete_leaf<Child: Height + GetHash>(
        &mut self,
        _index: u64,
        _leaf: &complete::Leaf<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit an internal node within the tree that is not on the frontier.
    fn complete_node<Child: Height + GetHash>(
        &mut self,
        _index: u64,
        _node: &complete::Node<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit the root of a complete tier within the tree.
    fn complete_tier<Child: Height + GetHash>(
        &mut self,
        _index: u64,
        _tier: &complete::Tier<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }

    /// Visit the top of a completed tree.
    fn complete_top<Child: Height + GetHash>(
        &mut self,
        _index: u64,
        _top: &complete::Top<Child>,
    ) -> Self::Output {
        Self::Output::default()
    }
}

impl<T: DefineVisitor> Visitor for T {
    type Output = T::Output;

    fn frontier_item(&mut self, index: u64, item: &frontier::Item) -> Self::Output {
        self.frontier_item(index, item)
    }

    fn frontier_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &frontier::Leaf<Child>,
    ) -> Self::Output {
        self.frontier_leaf(index, leaf)
    }

    fn frontier_node<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        node: &frontier::Node<Child>,
    ) -> Self::Output {
        self.frontier_node(index, node)
    }

    fn frontier_tier<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        tier: &frontier::Tier<Child>,
    ) -> Self::Output {
        self.frontier_tier(index, tier)
    }

    fn frontier_top<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        top: &frontier::Top<Child>,
    ) -> Self::Output {
        self.frontier_top(index, top)
    }

    fn complete_item(&mut self, index: u64, item: &complete::Item) -> Self::Output {
        self.complete_item(index, item)
    }

    fn complete_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &complete::Leaf<Child>,
    ) -> Self::Output {
        self.complete_leaf(index, leaf)
    }

    fn complete_node<Child: Height + GetHash>(
        &mut self,
        index: u64,
        node: &complete::Node<Child>,
    ) -> Self::Output {
        self.complete_node(index, node)
    }

    fn complete_tier<Child: Height + GetHash>(
        &mut self,
        index: u64,
        tier: &complete::Tier<Child>,
    ) -> Self::Output {
        self.complete_tier(index, tier)
    }

    fn complete_top<Child: Height + GetHash>(
        &mut self,
        index: u64,
        top: &complete::Top<Child>,
    ) -> Self::Output {
        self.complete_top(index, top)
    }
}
