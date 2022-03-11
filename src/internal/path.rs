//! An authentication path of a tree is a sequence of triples of hashes equal in length to the
//! height of the tree.
//!
//! The interpretation of an authentication path is dependent on an _index_ into the tree, stored
//! separately, which indicates the position of the leaf witnessed by the authentication path.

use crate::{
    internal::height::{IsHeight, Succ, Zero},
    Hash, Height,
};

/// An authentication path into a `Tree`.
///
/// This is statically guaranteed to have the same length as the height of the tree.
pub type AuthPath<Tree> = <<Tree as Height>::Height as Path>::Path;

/// Identifies the unique type representing an authentication path for the given height.
pub trait Path: IsHeight + Sized {
    /// The authentication path for this height.
    type Path;

    /// Calculate the root hash for a path leading to a leaf with the given index and hash.
    fn root(path: &Self::Path, index: u64, leaf: Hash) -> Hash;
}

/// The empty authentication path, for the zero-height tree.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Leaf;

impl Path for Zero {
    type Path = Leaf;

    #[inline]
    fn root(Leaf: &Leaf, _index: u64, leaf: Hash) -> Hash {
        leaf
    }
}

/// The authentication path for a node, whose height is always at least 1.
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
    fn root(Node { siblings, child }: &Node<Child>, index: u64, leaf: Hash) -> Hash {
        use WhichWay::*;

        // Based on the index, place the root hash of the child in the correct position among its
        // sibling hashes, so that we can hash this node
        let [leftmost, left, right, rightmost] = match (
            WhichWay::at(Self::HEIGHT, index), // Which child is the child?
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

impl WhichWay {
    /// Given a height and an index of a leaf, determine which direction the path down to that leaf
    /// should branch at the node at that height.
    #[inline]
    pub fn at(height: u64, index: u64) -> WhichWay {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    /// Get directions from the root (at the given height)
    fn directions_of_index(height: u64, index: u64) -> Vec<WhichWay> {
        (1..=height + 1)
            .rev() // iterate from the root to the leaf (height down to 1)
            .map(|height| WhichWay::at(height, index))
            .collect()
    }

    #[test]
    fn directions_of_index_check() {
        assert_eq!(directions_of_index(1, 0), &[WhichWay::Leftmost]);
        assert_eq!(directions_of_index(1, 1), &[WhichWay::Left]);
        assert_eq!(directions_of_index(1, 2), &[WhichWay::Right]);
        assert_eq!(directions_of_index(1, 3), &[WhichWay::Rightmost]);
    }

    /// Get the index which represents the given sequence of directions.
    fn index_of_directions(directions: &[WhichWay]) -> u64 {
        directions
            .iter()
            .rev() // Iterating rom the leaf to the root...
            .zip(1..) // Keeping track of the height (starting at 1 for the leafmost node)...
            .fold(0, |index, (&direction, height)| {
                index | // Set the bits in the index...
                (direction as u64) << (2 * (height - 1)) // ...which correspond to the direction at the height - 1.
            })
    }

    proptest! {
        #[test]
        fn which_way_correct(
        (height, index) in (
            // This is a dependent generator: we ensure that the index is in-bounds for the height
            (0u64..(3 * 8)), 0u64..u64::MAX).prop_map(|(height, index)| (height, (index % (4u64.pow(height as u32)))))
        ) {
            assert_eq!(index, index_of_directions(&directions_of_index(height, index)));
        }
    }
}
