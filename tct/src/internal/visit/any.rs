use super::*;

/// A representation of any node in the tree, not including its children or child hashes.
///
/// Any `T: Visit` can be converted `.into()` an [`Any`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Derivative)]
pub struct Any {
    /// The height of the node.
    pub height: u8,
    /// The index of the node from the lefthand side of the tree.
    ///
    /// For leaves at the bottom of the tree, this is equivalent to their position.
    pub index: u64,
    /// The kind of node.
    pub kind: Kind,
    /// The "place" of the node: whether or not it is on the frontier.
    pub place: Place,
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

/// The place a node is located in a tree: whether it is on the frontier or is completed.
///
/// This is redundant with the pair of (height, index) if the total size of the tree is known, but
/// it is useful to reveal it directly.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// The node is on the frontier.
    Frontier,
    /// The node is not on the frontier.
    Complete,
}

impl<T: Visit> From<T> for Any {
    fn from(node: T) -> Self {
        node.visit(&mut |any| any)
    }
}

impl<F, T> Visitor for F
where
    F: FnMut(Any) -> T,
{
    type Output = T;

    fn frontier_item(&mut self, index: u64, _item: &frontier::Item) -> Self::Output {
        self(Any {
            kind: Kind::Item,
            place: Place::Frontier,
            height: <frontier::Item as Height>::Height::HEIGHT,
            index,
        })
    }

    fn frontier_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        _leaf: &frontier::Leaf<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Leaf,
            place: Place::Frontier,
            height: <frontier::Leaf<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn frontier_node<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        _node: &frontier::Node<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Node,
            place: Place::Frontier,
            height: <frontier::Node<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn frontier_tier<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        _tier: &frontier::Tier<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Tier,
            place: Place::Frontier,
            height: <frontier::Tier<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn frontier_top<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        _top: &frontier::Top<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Top,
            place: Place::Frontier,
            height: <frontier::Top<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn complete_item(&mut self, index: u64, _item: &complete::Item) -> Self::Output {
        self(Any {
            kind: Kind::Item,
            place: Place::Complete,
            height: <complete::Item as Height>::Height::HEIGHT,
            index,
        })
    }

    fn complete_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        _leaf: &complete::Leaf<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Leaf,
            place: Place::Complete,
            height: <complete::Leaf<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn complete_node<Child: Height + GetHash>(
        &mut self,
        index: u64,
        _node: &complete::Node<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Node,
            place: Place::Complete,
            height: <complete::Node<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn complete_tier<Child: Height + GetHash>(
        &mut self,
        index: u64,
        _tier: &complete::Tier<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Tier,
            place: Place::Complete,
            height: <complete::Tier<Child> as Height>::Height::HEIGHT,
            index,
        })
    }

    fn complete_top<Child: Height + GetHash>(
        &mut self,
        index: u64,
        _top: &complete::Top<Child>,
    ) -> Self::Output {
        self(Any {
            kind: Kind::Top,
            place: Place::Complete,
            height: <complete::Top<Child> as Height>::Height::HEIGHT,
            index,
        })
    }
}
