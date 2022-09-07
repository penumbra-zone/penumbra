//! A dynamic representation of nodes within the internal tree structure.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use crate::prelude::*;

#[doc(inline)]
pub use crate::internal::hash::{Forgotten, Hash};

/// Every kind of node in the tree implements [`Node`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub(crate) trait Any<'tree>: GetHash + sealed::Sealed {
    /// The parent of this node, if any.
    ///
    /// This defaults to `None`, but is filled in by the [`Any`] implementation of [`Node`].
    fn parent<'a>(&'a self) -> Option<Node<'a, 'tree>> {
        None
    }

    /// The children of this node.
    fn children<'a>(&'a self) -> Vec<Node<'a, 'tree>>;

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    fn kind(&self) -> Kind;

    /// The most recent time something underneath this node was forgotten.
    fn forgotten(&self) -> Forgotten;

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    ///
    /// This defaults to 0, but is filled in by the [`Any`] implementation of [`Node`].
    fn index(&self) -> u64 {
        0
    }

    /// The position of the tree within which this node occurs.
    fn global_position(&self) -> Option<Position>;
}

impl GetHash for &dyn Any<'_> {
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }

    fn clear_cached_hash(&self) {
        (**self).clear_cached_hash()
    }
}

impl<'tree, T: Any<'tree>> Any<'tree> for &T {
    fn index(&self) -> u64 {
        (**self).index()
    }

    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn global_position(&self) -> Option<Position> {
        (**self).global_position()
    }

    fn parent(&self) -> Option<Node<'_, 'tree>> {
        (**self).parent()
    }

    fn forgotten(&self) -> Forgotten {
        (**self).forgotten()
    }

    fn children(&self) -> Vec<Node<'_, 'tree>> {
        (**self).children()
    }
}

/// The kind of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A leaf node at the bottom of the tree.
    Leaf {
        /// The witnessed commitment at this leaf, or `None` if this leaf was forgotten.
        commitment: Option<Commitment>,
    },
    /// An internal node within the tree.
    Internal {
        /// The height of this internal node.
        height: u8,
    },
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Leaf { .. } => write!(f, "Leaf",),
            Kind::Internal { .. } => write!(f, "Node"),
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

/// An arbitrary node somewhere within a tree.
#[derive(Copy, Clone)]
pub struct Node<'a, 'tree: 'a> {
    offset: u64,
    forgotten: Forgotten,
    parent: Option<&'a Node<'a, 'tree>>,
    this: Insert<&'a (dyn Any<'tree> + 'a)>,
}

impl Debug for Node<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{}::{}", self.place(), self.kind());
        let mut s = f.debug_struct(&name);
        if self.height() != 0 {
            s.field("height", &(*self).height());
        }
        s.field("position", &u64::from(self.position()));
        if self.forgotten() != Forgotten::default() {
            s.field("forgotten", &self.forgotten());
        }
        if let Some(hash) = self.cached_hash() {
            s.field("hash", &hash);
        }
        if let Kind::Leaf {
            commitment: Some(commitment),
        } = self.kind()
        {
            s.field("commitment", &commitment);
        }
        let children = self.children();
        if !children.is_empty() {
            s.field("children", &children);
        }
        s.finish()
    }
}

impl Display for Node<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("position", &self.position())
            .finish_non_exhaustive()
    }
}

impl<'a, 'tree: 'a> Node<'a, 'tree> {
    /// Make a root node.
    pub(crate) fn root(this: &'a dyn Any<'tree>) -> Self {
        Self {
            offset: 0,
            forgotten: this.forgotten(),
            parent: None,
            this: Insert::Keep(this),
        }
    }

    /// Make a new [`Child`] from a reference to something implementing [`Node`].
    pub(crate) fn child(forgotten: Forgotten, child: Insert<&'a dyn Any<'tree>>) -> Self {
        Node {
            offset: 0,
            forgotten,
            parent: None,
            this: child,
        }
    }

    /// The parent of this node, if any.
    pub fn parent(&'a self) -> Option<Node<'a, 'tree>> {
        (self as &dyn Any<'tree>).parent()
    }

    /// The children of this node.
    pub fn children(&'a self) -> Vec<Node<'a, 'tree>> {
        (self as &dyn Any<'tree>).children()
    }

    /// The hash of this node.
    pub fn hash(&self) -> Hash {
        self.this.hash()
    }

    /// The cached hash at this node, if any.
    pub fn cached_hash(&self) -> Option<Hash> {
        self.this.cached_hash()
    }

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    pub fn kind(&self) -> Kind {
        (self as &dyn Any).kind()
    }

    /// The most recent time something underneath this node was forgotten.
    pub fn forgotten(&self) -> Forgotten {
        (self as &dyn Any).forgotten()
    }

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    pub fn index(&self) -> u64 {
        (self as &dyn Any).index()
    }

    /// The height of this node above the base of the tree.
    pub fn height(&self) -> u8 {
        match self.kind() {
            Kind::Internal { height } => height,
            Kind::Leaf { .. } => 0,
        }
    }

    /// The position of the node (the vertical extension of the position of commitments below).
    pub fn position(&self) -> Position {
        (4u64.pow(self.height() as u32) * self.index()).into()
    }

    /// The distance between positions of nodes at this height.
    pub fn stride(&self) -> u64 {
        4u64.pow(self.height() as u32)
    }

    /// The range of positions that could occur beneath this node (not all of them need be actually
    /// represented in the tree).
    pub fn range(&self) -> Range<Position> {
        let position: u64 = self.position().into();
        position.into()..(position + self.stride()).min(4u64.pow(24) - 1).into()
    }

    /// The place on the tree where this node occurs.
    pub fn place(&self) -> Place {
        if let Some(global_position) = self.global_position() {
            if let Some(frontier_tip) = u64::from(global_position).checked_sub(1) {
                let height = self.height();
                let position = u64::from(self.position());
                if position >> (height * 2) == frontier_tip >> (height * 2) {
                    // The prefix of the position down to this height matches the path to the
                    // frontier tip, which means that this node is on the frontier
                    Place::Frontier
                } else {
                    // The prefix doesn't match (the node's position is less than the frontier, even
                    // considering only the prefix), so it's complete
                    Place::Complete
                }
            } else {
                // The global position is zero (the tree is empty, i.e. all nodes, that is, none,
                // are frontier nodes, vacuously)
                Place::Frontier
            }
        } else {
            // There is no global position (the tree is full, i.e. all nodes are complete)
            Place::Complete
        }
    }
}

impl GetHash for Node<'_, '_> {
    fn hash(&self) -> Hash {
        self.this.hash()
    }

    fn cached_hash(&self) -> Option<Hash> {
        self.this.cached_hash()
    }
}

impl<'a, 'tree: 'a> Any<'tree> for Node<'a, 'tree> {
    fn index(&self) -> u64 {
        self.offset
    }

    fn parent(&self) -> Option<Node<'a, 'tree>> {
        self.parent.copied()
    }

    fn kind(&self) -> Kind {
        match self.this {
            Insert::Keep(child) => child.kind(),
            Insert::Hash(_) => {
                if let Some(parent) = self.parent {
                    match parent.kind() {
                        Kind::Internal {
                            height: height @ 2..=24,
                        } => Kind::Internal { height: height - 1 },
                        Kind::Internal { height: 1 } => Kind::Leaf { commitment: None },
                        Kind::Internal {
                            height: 0 | 25..=u8::MAX,
                        } => {
                            unreachable!("nodes cannot have zero height or height greater than 24")
                        }
                        Kind::Leaf { .. } => unreachable!("leaves cannot have children"),
                    }
                } else {
                    // Hashed root node is an internal node of height 24
                    Kind::Internal { height: 24 }
                }
            }
        }
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten
    }

    fn global_position(&self) -> Option<Position> {
        if let Some(parent) = self.parent {
            parent.global_position()
        } else {
            self.this.keep().and_then(Any::global_position)
        }
    }

    fn children(&self) -> Vec<Node<'_, 'tree>> {
        if let Insert::Keep(child) = self.this {
            child
                .children()
                .into_iter()
                .enumerate()
                .map(|(nth, child)| {
                    debug_assert_eq!(
                        child.offset, 0,
                        "explicitly constructed children should have zero offset"
                    );
                    Node {
                        forgotten: child.forgotten,
                        this: child.this,
                        parent: Some(self),
                        offset: self.offset * 4 + nth as u64,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

#[doc(inline)]
pub use traverse::{traverse, traverse_async};

/// Functions to perform traversals of [`Node`]s in synchronous and asynchronous contexts.
pub mod traverse {
    use std::{future::Future, pin::Pin};

    use super::Node;

    /// Flag to determine whether the traversal continues downward from this node or stops at the
    /// current node.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Recur {
        /// Default: Continue downwards.
        Down,
        /// Continue downwards, but traversing children right-to-left.
        DownBackwards,
        /// Stop here and don't continue downwards.
        Stop,
    }

    impl Default for Recur {
        fn default() -> Self {
            Down
        }
    }

    impl From<()> for Recur {
        fn from(_: ()) -> Self {
            Down
        }
    }

    #[doc(inline)]
    pub use Recur::*;

    /// Synchronously traverse a node depth-first, visiting each node using the given function.
    ///
    /// If the function returns [`Stop`], the traversal will stop at this node and not continue
    /// into the children; otherwise it will recur downwards.
    ///
    /// Note that because `(): Into<Recur>`, it is valid to pass a function which returns `()` if
    /// the traversal should always recur and never stop before reaching a leaf.
    pub fn traverse<R: Into<Recur>>(node: Node, with: &mut impl FnMut(Node) -> R) {
        if let down @ (Down | DownBackwards) = with(node).into() {
            let mut children = node.children();
            if let DownBackwards = down {
                children.reverse();
            }
            for child in children {
                traverse(child, with);
            }
        }
    }

    /// Asynchronously traverse a node depth-first, visiting each node using the given function.
    ///
    /// If the function returns [`Stop`], the traversal will stop at this node and not continue
    /// into the children; otherwise it will recur downwards.
    ///
    /// Note that because `(): Into<Recur>`, it is valid to pass a function which returns `()` if
    /// the traversal should always recur and never stop before reaching a leaf.
    pub async fn traverse_async<'a, 'tree: 'a, R: Send + Into<Recur>, Fut>(
        node: Node<'a, 'tree>,
        with: &mut (impl FnMut(Node) -> Fut + Send),
    ) where
        Fut: Future<Output = R> + Send,
    {
        // We need this inner function to correctly specify the lifetimes for the recursive call
        fn traverse_async_inner<'a, 'tree: 'a, 'b: 'a, R: Send + Into<Recur>, F, Fut>(
            node: Node<'a, 'tree>,
            with: &'b mut F,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
        where
            F: FnMut(Node) -> Fut + Send,
            Fut: Future<Output = R> + Send,
        {
            Box::pin(async move {
                if let down @ (Down | DownBackwards) = with(node).await.into() {
                    let mut children = node.children();
                    if let DownBackwards = down {
                        children.reverse();
                    }
                    for child in children {
                        traverse_async_inner::<'_, '_, '_, R, F, Fut>(child, with).await;
                    }
                }
            })
        }

        traverse_async_inner::<'_, '_, '_, R, _, Fut>(node, with).await
    }
}

mod sealed {
    use super::*;

    pub trait Sealed: Send + Sync {}

    impl<T: Sealed> Sealed for &T {}
    impl Sealed for Node<'_, '_> {}

    impl Sealed for complete::Item {}
    impl<T: Sealed> Sealed for complete::Leaf<T> {}
    impl<T: Sealed> Sealed for complete::Node<T> {}
    impl<T: Sealed + Height + GetHash> Sealed for complete::Tier<T> {}
    impl<T: Sealed + Height + GetHash> Sealed for complete::Top<T> {}

    impl Sealed for frontier::Item {}
    impl<T: Sealed> Sealed for frontier::Leaf<T> {}
    impl<T: Sealed + Focus> Sealed for frontier::Node<T> where T::Complete: Send + Sync {}
    impl<T: Sealed + Height + GetHash + Focus> Sealed for frontier::Tier<T> where
        T::Complete: Send + Sync
    {
    }
    impl<T: Sealed + Height + GetHash + Focus> Sealed for frontier::Top<T> where T::Complete: Send + Sync
    {}
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

        fn check_leaves(index: &mut [u64; 9], node: Node) {
            assert_eq!(node.index(), index[usize::from(node.height())], "{}", node);

            index[usize::from(node.height())] += 1;

            for child in node.children() {
                check_leaves(index, child);
            }
        }

        check_leaves(&mut [0; 9], Node::root(&top));
    }

    #[test]
    fn parent_of_child() {
        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        top.insert(Commitment(0u8.into()).into()).unwrap();

        // This can't be a loop, it has to be recursive because the lifetime parameter of the parent
        // is different for each recursive call
        fn check(parent: Node) {
            if let Some(child) = parent.children().pop() {
                let parent_of_child = child.parent().expect("child has no parent");
                assert_eq!(
                    parent.hash(),
                    parent_of_child.hash(),
                    "parent hash mismatch"
                );
                check(child);
            } else {
                assert_eq!(parent.height(), 0, "got all the way to a leaf");
            }
        }

        let root = Node::root(&top);
        check(root);
    }

    #[test]
    fn place_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        for i in 0..MAX_SIZE_TO_TEST {
            top.insert(Commitment(i.into()).into()).unwrap();
        }

        fn check(node: Node, expected: Place) {
            assert_eq!(node.place(), expected);
            match node.children().as_slice() {
                [] => {}
                [a] => {
                    check(*a, expected);
                }
                [a, b] => {
                    check(*a, Place::Complete);
                    check(*b, expected);
                }
                [a, b, c] => {
                    check(*a, Place::Complete);
                    check(*b, Place::Complete);
                    check(*c, expected);
                }
                [a, b, c, d] => {
                    check(*a, Place::Complete);
                    check(*b, Place::Complete);
                    check(*c, Place::Complete);
                    check(*d, expected);
                }
                _ => unreachable!("nodes can't have > 4 children"),
            }
        }

        let root = Node::root(&top);
        check(root, Place::Frontier);
    }
}
