use std::fmt::Display;

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// The node is on the frontier.
    Frontier,
    /// The node is not on the frontier.
    Complete,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Frontier => write!(f, "frontier"),
            Place::Complete => write!(f, "complete"),
        }
    }
}

impl<T: Visit> From<T> for Any {
    fn from(node: T) -> Self {
        node.visit(&mut AnyVisitor(|any, _: &dyn Visit| any))
    }
}

/// A wrapper for a visitor defined in terms of a function on `Any`.
///
/// This struct is a [`Visitor`] if the wrapped thing is `FnMut(Any) -> T`, for any `T`.
pub struct AnyVisitor<F>(pub F);

impl<F, T> Visitor for AnyVisitor<F>
where
    F: FnMut(Any, &dyn Visit) -> T,
{
    type Output = T;

    fn frontier_item(&mut self, index: u64, item: &frontier::Item) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Item,
                place: Place::Frontier,
                height: <frontier::Item as Height>::Height::HEIGHT,
                index,
            },
            item,
        )
    }

    fn frontier_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &frontier::Leaf<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Leaf,
                place: Place::Frontier,
                height: <frontier::Leaf<Child> as Height>::Height::HEIGHT,
                index,
            },
            leaf,
        )
    }

    fn frontier_node<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        node: &frontier::Node<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Node,
                place: Place::Frontier,
                height: <frontier::Node<Child> as Height>::Height::HEIGHT,
                index,
            },
            node,
        )
    }

    fn frontier_tier<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        tier: &frontier::Tier<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Tier,
                place: Place::Frontier,
                height: <frontier::Tier<Child> as Height>::Height::HEIGHT,
                index,
            },
            tier,
        )
    }

    fn frontier_top<Child: Height + Focus + GetHash>(
        &mut self,
        index: u64,
        top: &frontier::Top<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Top,
                place: Place::Frontier,
                height: <frontier::Top<Child> as Height>::Height::HEIGHT,
                index,
            },
            top,
        )
    }

    fn complete_item(&mut self, index: u64, item: &complete::Item) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Item,
                place: Place::Complete,
                height: <complete::Item as Height>::Height::HEIGHT,
                index,
            },
            item,
        )
    }

    fn complete_leaf<Child: Height + GetHash>(
        &mut self,
        index: u64,
        leaf: &complete::Leaf<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Leaf,
                place: Place::Complete,
                height: <complete::Leaf<Child> as Height>::Height::HEIGHT,
                index,
            },
            leaf,
        )
    }

    fn complete_node<Child: Height + GetHash>(
        &mut self,
        index: u64,
        node: &complete::Node<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Node,
                place: Place::Complete,
                height: <complete::Node<Child> as Height>::Height::HEIGHT,
                index,
            },
            node,
        )
    }

    fn complete_tier<Child: Height + GetHash>(
        &mut self,
        index: u64,
        tier: &complete::Tier<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Tier,
                place: Place::Complete,
                height: <complete::Tier<Child> as Height>::Height::HEIGHT,
                index,
            },
            tier,
        )
    }

    fn complete_top<Child: Height + GetHash>(
        &mut self,
        index: u64,
        top: &complete::Top<Child>,
    ) -> Self::Output {
        self.0(
            Any {
                kind: Kind::Top,
                place: Place::Complete,
                height: <complete::Top<Child> as Height>::Height::HEIGHT,
                index,
            },
            top,
        )
    }
}
