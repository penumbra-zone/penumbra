//! An authentication path of a tree is a sequence of triples of hashes equal in length to the
//! height of the tree.
//!
//! The interpretation of an authentication path is dependent on an _index_ into the tree, stored
//! separately, which indicates the position of the leaf witnessed by the authentication path.

use crate::{
    internal::height::{IsHeight, Succ, Zero},
    Hash,
};

/// Identifies the unique type representing an authentication path for the given height.
pub trait Path: IsHeight + Sized {
    /// The authentication path for this height.
    type Path;

    /// Calculate the root hash for a path leading to a leaf with the given index and hash.
    fn root(path: &Self::Path, index: usize, leaf: Hash) -> Hash;
}

/// The empty authentication path, for the zero-height tree.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Leaf;

impl Path for Zero {
    type Path = Leaf;

    #[inline]
    fn root(Leaf: &Leaf, _index: usize, leaf: Hash) -> Hash {
        leaf
    }
}

/// The authentication path for a node
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Node<Child> {
    /// The sibling hashes of the child.
    ///
    /// Note that this does not record which child is witnessed; that information lies in the index
    /// of the leaf.
    pub siblings: [Hash; 3],

    /// The authentication path for the witnessed child.
    pub child: Child,
}

impl<Child, N: Path<Path = Child>> Path for Succ<N> {
    type Path = Node<Child>;

    #[inline]
    #[rustfmt::skip] // For reading clarity, this function is laid out very carefully
    fn root(Node { siblings, child }: &Node<Child>, index: usize, leaf: Hash) -> Hash {
        use WhichWay::*;

        // Based on the index, place the root hash of the child in the correct position among its
        // sibling hashes, so that we can hash this node
        let [leftmost, left, right, rightmost] = match (
            which_way(Self::HEIGHT, index), // Which child is the child?
            N::root(child, index, leaf),    // The root hash down to the leaf from the child
            *siblings,                      // The hashes of the siblings of the child
        ) {
            (Leftmost,  leftmost,  [/* leftmost, */ left,    right,    rightmost   ]) |
            (Left,      left,      [   leftmost, /* left, */ right,    rightmost   ]) |
            (Right,     right,     [   leftmost,    left, /* right, */ rightmost   ]) |
            (Rightmost, rightmost, [   leftmost,    left,    right, /* rightmost */])
	                        => [   leftmost,    left,    right,    rightmost   ],
        };

        // Get the hash of this node at its correct height
        Hash::node(Self::HEIGHT, leftmost, left, right, rightmost)
    }
}

/// An enumeration of the different ways a path can go down a quadtree.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WhichWay {
    /// The leftmost (0th) child.
    Leftmost,
    /// The left (1st) child.
    Left,
    /// The right (2nd) child.
    Right,
    /// The rightmost (3rd) child.
    Rightmost,
}

/// Given a height and an index of a leaf, determine which direction the path down to that leaf
/// should branch at the node at that height.
#[inline]
pub fn which_way(height: usize, index: usize) -> WhichWay {
    // Shift the index right by (2 * (height - 1)) so that the last 2 bits are our direction, then
    // mask off just those bits and branch on them to generate the output
    match (index >> (2 * (height - 1))) & 0b11 {
        0 => WhichWay::Leftmost,
        1 => WhichWay::Left,
        2 => WhichWay::Right,
        3 => WhichWay::Rightmost,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn which_way_works() {}
}
