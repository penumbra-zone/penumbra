//! A dynamic representation of nodes within the internal tree structure.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use archery::{SharedPointer, SharedPointerKind};

use crate::prelude::*;

#[doc(inline)]
pub use crate::internal::hash::{Forgotten, Hash};

/// Every kind of node in the tree implements [`Node`], and its methods collectively describe every
/// salient fact about each node, dynamically rather than statically as in the rest of the crate.
pub(crate) trait Any<RefKind: SharedPointerKind>: GetHash + sealed::Sealed {
    /// The children of this node.
    fn children(&self) -> Vec<Node<RefKind>>;

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

impl<RefKind: SharedPointerKind> GetHash for &dyn Any<RefKind> {
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

impl<T: Any<RefKind>, RefKind: SharedPointerKind> Any<RefKind> for &T {
    fn index(&self) -> u64 {
        (**self).index()
    }

    fn kind(&self) -> Kind {
        (**self).kind()
    }

    fn global_position(&self) -> Option<Position> {
        (**self).global_position()
    }

    fn forgotten(&self) -> Forgotten {
        (**self).forgotten()
    }

    fn children(&self) -> Vec<Node<RefKind>> {
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
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Node<RefKind: SharedPointerKind = archery::ArcK> {
    offset: u64,
    forgotten: Forgotten,
    global_position: Option<Position>,
    kind: Kind,
    this: Insert<SharedPointer<Box<dyn Any<RefKind>>, RefKind>>,
}

impl<RefKind: SharedPointerKind> Debug for Node<RefKind> {
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

impl<RefKind: SharedPointerKind> Display for Node<RefKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{}::{}", self.place(), self.kind()))
            .field("height", &self.height())
            .field("position", &self.position())
            .finish_non_exhaustive()
    }
}

impl<RefKind: SharedPointerKind> Node<RefKind> {
    /// Make a root node.
    pub(crate) fn root(this: Box<dyn Any<RefKind>>) -> Self {
        Self {
            offset: 0,
            forgotten: this.forgotten(),
            global_position: this.global_position(),
            kind: this.kind(),
            this: Insert::Keep(SharedPointer::new(this)),
        }
    }

    /// Make a new [`Child`] from a reference to something implementing [`Node`].
    pub(crate) fn child(
        parent: &dyn Any<RefKind>,
        forgotten: Forgotten,
        child: Insert<Box<dyn Any<RefKind>>>,
    ) -> Self {
        Node {
            offset: 0,
            forgotten,
            global_position: todo!("implement global position"),
            kind: match child {
                Insert::Keep(ref child) => child.kind(),
                Insert::Hash(_) => match parent.kind() {
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
                },
            },
            this: child.map(SharedPointer::new),
        }
    }

    /// The children of this node.
    pub fn children(&self) -> Vec<Node<RefKind>> {
        (self as &dyn Any<RefKind>).children()
    }

    /// The hash of this node.
    pub fn hash(&self) -> Hash {
        match self.this {
            Insert::Keep(ref this) => this.hash(),
            Insert::Hash(ref hash) => *hash,
        }
    }

    /// The cached hash at this node, if any.
    pub fn cached_hash(&self) -> Option<Hash> {
        match self.this {
            Insert::Keep(ref this) => this.cached_hash(),
            Insert::Hash(ref hash) => Some(*hash),
        }
    }

    /// The kind of the node: either a [`Kind::Internal`] with a height, or a [`Kind::Leaf`] with an
    /// optional [`Commitment`].
    pub fn kind(&self) -> Kind {
        (self as &dyn Any<RefKind>).kind()
    }

    /// The most recent time something underneath this node was forgotten.
    pub fn forgotten(&self) -> Forgotten {
        (self as &dyn Any<RefKind>).forgotten()
    }

    /// The index of this node from the left of the tree.
    ///
    /// For items at the base, this is the position of the item.
    pub fn index(&self) -> u64 {
        (self as &dyn Any<RefKind>).index()
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

    /// The global position of the tree inside of which this node exists.
    pub fn global_position(&self) -> Option<Position> {
        <Self as Any<RefKind>>::global_position(self)
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

    /// Given a function to manipulate an in-progress traversal and return elements of an iterator,
    /// create that iterator.
    ///
    /// **Important:** If you want the iterator to end, you must call [`Traverse::stop`]; otherwise,
    /// the returned iterator will loop forever.
    pub fn traverse<F>(self, with: F) -> Traversal<F, RefKind> {
        Traversal {
            traversal: Traverse::new(self),
            with,
        }
    }
}

impl<RefKind: SharedPointerKind> GetHash for Node<RefKind> {
    fn hash(&self) -> Hash {
        match self.this {
            Insert::Keep(ref this) => this.hash(),
            Insert::Hash(ref hash) => *hash,
        }
    }

    fn cached_hash(&self) -> Option<Hash> {
        match self.this {
            Insert::Keep(ref this) => this.cached_hash(),
            Insert::Hash(ref hash) => Some(*hash),
        }
    }
}

impl<RefKind: SharedPointerKind> Any<RefKind> for Node<RefKind> {
    fn index(&self) -> u64 {
        self.offset
    }

    fn kind(&self) -> Kind {
        self.kind
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten
    }

    fn global_position(&self) -> Option<Position> {
        self.global_position
    }

    fn children(&self) -> Vec<Node<RefKind>> {
        if let Insert::Keep(child) = self.this.as_ref() {
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
                        kind: child.kind,
                        global_position: child.global_position,
                        offset: self.offset * 4 + nth as u64,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

/// An in-progress traversal, which can be moved around and inspected.
#[derive(Clone, Debug)]
pub struct Traverse<RefKind: SharedPointerKind = archery::ArcK> {
    above: Vec<Siblings<RefKind>>,
    here: Siblings<RefKind>,
    stop: bool,
}

#[derive(Clone, Debug)]
struct Siblings<RefKind: SharedPointerKind = archery::ArcK> {
    left: Vec<Node<RefKind>>,
    here: Node<RefKind>,
    right: Vec<Node<RefKind>>,
}

/// An iterator over some part of a tree, defined by an arbitrary function manipulating an
/// in-progress traversal.
#[derive(Clone, Debug)]
pub struct Traversal<F, RefKind: SharedPointerKind = archery::ArcK> {
    traversal: Traverse<RefKind>,
    with: F,
}

impl<F, T, RefKind: SharedPointerKind> Iterator for Traversal<F, RefKind>
where
    F: FnMut(&mut Traverse<RefKind>) -> Option<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.traversal.stop {
                if let Some(output) = (self.with)(&mut self.traversal) {
                    break Some(output);
                }
            } else {
                break None;
            }
        }
    }
}

impl<RefKind: SharedPointerKind> Traverse<RefKind> {
    fn new(root: Node<RefKind>) -> Self {
        Self {
            above: vec![],
            here: Siblings {
                left: vec![],
                here: root,
                right: vec![],
            },
            stop: false,
        }
    }

    /// Get the node here.
    pub fn here(&self) -> &Node<RefKind> {
        &self.here.here
    }

    /// Stop iterating after this next element is returned.
    pub fn stop(&mut self) {
        self.stop = true;
    }

    /// Move up by one level in the tree, or return `false`.
    pub fn up(&mut self) -> bool {
        if let Some(above) = self.above.pop() {
            self.here = above;
            true
        } else {
            false
        }
    }

    /// Move down by one level in the tree, starting at the left-hand child, or return `false`.
    pub fn down(&mut self) -> bool {
        let mut children = self.here.here.children();
        if let Some(here) = children.pop() {
            let siblings = Siblings {
                left: vec![],
                here,
                right: children,
            };
            self.above.push(std::mem::replace(&mut self.here, siblings));
            true
        } else {
            false
        }
    }

    /// Move to the left sibling, or return `false`.
    pub fn left(&mut self) -> bool {
        if let Some(left) = self.here.left.pop() {
            self.here
                .right
                .push(std::mem::replace(&mut self.here.here, left));
            true
        } else {
            false
        }
    }

    /// Move to the right sibling, or return `false`.
    pub fn right(&mut self) -> bool {
        if let Some(right) = self.here.right.pop() {
            self.here
                .left
                .push(std::mem::replace(&mut self.here.here, right));
            true
        } else {
            false
        }
    }

    /// Move to the next right sibling or ancestor, going up as many levels as necessary, or return `false`.
    pub fn next_right(&mut self) -> bool {
        self.right()
            || loop {
                if !self.up() {
                    break false;
                } else if self.right() {
                    break true;
                }
            }
    }

    /// Move to the next left sibling or ancestor, going up as many levels as necessary, or return `false`.
    pub fn next_left(&mut self) -> bool {
        self.left()
            || loop {
                if !self.up() {
                    break false;
                } else if self.left() {
                    break true;
                }
            }
    }
}

mod sealed {
    use archery::SharedPointerKind;

    use super::*;

    pub trait Sealed {}

    impl<T: Sealed> Sealed for &T {}
    impl<RefKind: SharedPointerKind> Sealed for Node<RefKind> {}

    impl Sealed for complete::Item {}
    impl<T: Sealed> Sealed for complete::Leaf<T> {}
    impl<T: Sealed + Clone, RefKind: SharedPointerKind> Sealed for complete::Node<T, RefKind> {}
    impl<T: Sealed + Height + GetHash + Clone, RefKind: SharedPointerKind> Sealed
        for complete::Tier<T, RefKind>
    {
    }
    impl<T: Sealed + Height + GetHash + Clone, RefKind: SharedPointerKind> Sealed
        for complete::Top<T, RefKind>
    {
    }

    impl Sealed for frontier::Item {}
    impl<T: Sealed> Sealed for frontier::Leaf<T> {}
    impl<T: Sealed + Focus, RefKind: SharedPointerKind> Sealed for frontier::Node<T, RefKind> {}
    impl<T: Sealed + Height + GetHash + Focus + Clone, RefKind: SharedPointerKind> Sealed
        for frontier::Tier<T, RefKind>
    where
        T::Complete: Clone,
    {
    }
    impl<T: Sealed + Height + GetHash + Focus + Clone, RefKind: SharedPointerKind> Sealed
        for frontier::Top<T, RefKind>
    where
        T::Complete: Clone,
    {
    }
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

        check_leaves(&mut [0; 9], Node::root(Box::new(top)));
    }

    #[test]
    fn place_correct() {
        const MAX_SIZE_TO_TEST: u16 = 100;

        let mut top: frontier::Top<Item> = frontier::Top::new(frontier::TrackForgotten::No);
        for i in 0..MAX_SIZE_TO_TEST {
            assert_eq!(top.global_position(), Some(u64::from(i).into()));
            top.insert(Commitment(i.into()).into()).unwrap();
        }

        fn check(node: Node, expected: Place) {
            assert_eq!(
                node.global_position(),
                Some(u64::from(MAX_SIZE_TO_TEST).into()),
                "{}",
                node
            );
            assert_eq!(
                node.place(),
                expected,
                "node: {}, global position: {:?}",
                node,
                node.global_position()
            );
            match node.children().as_slice() {
                [] => {}
                [a] => {
                    check(a.clone(), expected);
                }
                [a, b] => {
                    check(a.clone(), Place::Complete);
                    check(b.clone(), expected);
                }
                [a, b, c] => {
                    check(a.clone(), Place::Complete);
                    check(b.clone(), Place::Complete);
                    check(c.clone(), expected);
                }
                [a, b, c, d] => {
                    check(a.clone(), Place::Complete);
                    check(b.clone(), Place::Complete);
                    check(c.clone(), Place::Complete);
                    check(d.clone(), expected);
                }
                _ => unreachable!("nodes can't have > 4 children"),
            }
        }

        let root = Node::root(Box::new(top));
        check(root, Place::Frontier);
    }
}
