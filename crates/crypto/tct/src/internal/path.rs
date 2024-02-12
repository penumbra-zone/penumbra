//! Authentication paths into the tree, generically for trees of any height.
//!
//! An authentication path of a tree is a sequence of triples of hashes equal in length to the
//! height of the tree.
//!
//! The interpretation of an authentication path is dependent on an _index_ into the tree, stored
//! separately, which indicates the position of the leaf witnessed by the authentication path.
//!
//! These are wrapped in more specific domain types by the exposed crate API to make it more
//! comprehensible.

use crate::prelude::*;

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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    fn root(Node { siblings, child }: &Node<Child>, index: u64, leaf: Hash) -> Hash {
        // Based on the index, place the root hash of the child in the correct position among its
        // sibling hashes, so that we can hash this node
        let which_way = WhichWay::at(Self::HEIGHT, index).0;
        let [leftmost, left, right, rightmost] =
            which_way.insert(N::root(child, index, leaf), *siblings);

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
    pub fn at(height: u8, index: u64) -> (WhichWay, u64) {
        // Shift the index right by (2 * (height - 1)) so that the last 2 bits are our direction, then
        // mask off just those bits and branch on them to generate the output
        let which_way = match (index >> (2 * (height - 1))) & 0b11 {
            0 => WhichWay::Leftmost,
            1 => WhichWay::Left,
            2 => WhichWay::Right,
            3 => WhichWay::Rightmost,
            _ => unreachable!(),
        };

        // The index into the child: mask off the bits we just used to determine the direction
        let index = index & !(0b11 << ((height - 1) * 2));

        (which_way, index)
    }

    /// Given a 3-element array, insert an item into the array in the place indicated by the [`WhichWay`].
    ///
    /// This is the inverse of [`WhichWay::pick`].
    #[inline]
    pub fn insert<T>(&self, item: T, siblings: [T; 3]) -> [T; 4] {
        use WhichWay::*;

        let (
            (Leftmost,  leftmost,  [/* leftmost, */ left,    right,    rightmost   ]) |
            (Left,      left,      [   leftmost, /* left, */ right,    rightmost   ]) |
            (Right,     right,     [   leftmost,    left, /* right, */ rightmost   ]) |
            (Rightmost, rightmost, [   leftmost,    left,    right, /* rightmost */])
        ) = (self, item, siblings);

        [leftmost, left, right, rightmost]
    }

    /// Given a 4-element array, pick out the item in the array indicated by the [`WhichWay`], and
    /// pair it with all the others, in the order they occurred.
    ///
    /// This is the inverse of [`WhichWay::insert`].
    #[inline]
    pub fn pick<T>(&self, siblings: [T; 4]) -> (T, [T; 3]) {
        use WhichWay::*;

        let ((Leftmost, [picked, a, b, c])
        | (Left, [a, picked, b, c])
        | (Right, [a, b, picked, c])
        | (Rightmost, [a, b, c, picked])) = (self, siblings);

        (picked, [a, b, c])
    }
}

impl<T> Index<WhichWay> for [T; 4] {
    type Output = T;

    fn index(&self, index: WhichWay) -> &T {
        match index {
            WhichWay::Leftmost => &self[0],
            WhichWay::Left => &self[1],
            WhichWay::Right => &self[2],
            WhichWay::Rightmost => &self[3],
        }
    }
}

impl<T> IndexMut<WhichWay> for [T; 4] {
    fn index_mut(&mut self, index: WhichWay) -> &mut T {
        match index {
            WhichWay::Leftmost => &mut self[0],
            WhichWay::Left => &mut self[1],
            WhichWay::Right => &mut self[2],
            WhichWay::Rightmost => &mut self[3],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    /// Get directions from the root (at the given height)
    fn directions_of_index(height: u8, index: u64) -> Vec<WhichWay> {
        (1..=height)
            .rev() // iterate from the root to the leaf (height down to 1)
            .map(|height| WhichWay::at(height, index).0)
            .collect()
    }

    /// Get a sequence of indices representing the index of the originally specified leaf from the
    /// starting height down to zero.
    fn directions_via_indices(height: u8, index: u64) -> Vec<WhichWay> {
        (1..=height)
            .rev() // iterate from the leaf to the root (height down to 1)
            .scan(index, |index, height| {
                let (which_way, next_index) = WhichWay::at(height, *index);
                *index = next_index;
                Some(which_way)
            })
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
        fn which_way_indices_correct(
            (height, index) in (
                // This is a dependent generator: we ensure that the index is in-bounds for the height
                (0u8..(3 * 8)), 0u64..u64::MAX).prop_map(|(height, index)| (height, (index % (4u64.pow(height as u32))))
            )
        ) {
            assert_eq!(directions_of_index(height, index), directions_via_indices(height, index));
        }

        #[test]
        fn which_way_direction_correct(
            (height, index) in (
                // This is a dependent generator: we ensure that the index is in-bounds for the height
                (0u8..(3 * 8)), 0u64..u64::MAX).prop_map(|(height, index)| (height, (index % (4u64.pow(height as u32)))))
        ) {
            assert_eq!(index, index_of_directions(&directions_of_index(height, index)));
        }
    }
}

// All the below is just for serialization to/from protobufs:

/// When deserializing an authentication path, it was malformed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
#[error("could not decode authentication path")]
pub struct PathDecodeError;

use decaf377::Fq;
use penumbra_proto::penumbra::crypto::tct::v1 as pb;
use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

impl From<Leaf> for VecDeque<pb::MerklePathChunk> {
    fn from(Leaf: Leaf) -> VecDeque<pb::MerklePathChunk> {
        VecDeque::new()
    }
}

impl From<Leaf> for Vec<pb::MerklePathChunk> {
    fn from(Leaf: Leaf) -> Vec<pb::MerklePathChunk> {
        Vec::new()
    }
}

impl TryFrom<VecDeque<pb::MerklePathChunk>> for Leaf {
    type Error = PathDecodeError;

    fn try_from(queue: VecDeque<pb::MerklePathChunk>) -> Result<Leaf, Self::Error> {
        if queue.is_empty() {
            Ok(Leaf)
        } else {
            Err(PathDecodeError)
        }
    }
}

impl TryFrom<Vec<pb::MerklePathChunk>> for Leaf {
    type Error = PathDecodeError;

    fn try_from(vec: Vec<pb::MerklePathChunk>) -> Result<Leaf, Self::Error> {
        if vec.is_empty() {
            Ok(Leaf)
        } else {
            Err(PathDecodeError)
        }
    }
}

// To create `Vec<pb::MerklePathChunk>`, we have a recursive impl for `VecDeque` which we delegate
// to, then finally turn into a `Vec` at the end.
impl<Child> From<Node<Child>> for VecDeque<pb::MerklePathChunk>
where
    VecDeque<pb::MerklePathChunk>: From<Child>,
{
    fn from(node: Node<Child>) -> VecDeque<pb::MerklePathChunk> {
        let [sibling_1, sibling_2, sibling_3] =
            node.siblings.map(|hash| Fq::from(hash).to_bytes().to_vec());
        let mut path: VecDeque<pb::MerklePathChunk> = node.child.into();
        path.push_front(pb::MerklePathChunk {
            sibling_1,
            sibling_2,
            sibling_3,
        });
        path
    }
}

impl<Child> From<Node<Child>> for Vec<pb::MerklePathChunk>
where
    VecDeque<pb::MerklePathChunk>: From<Child>,
{
    fn from(node: Node<Child>) -> Vec<pb::MerklePathChunk> {
        let [sibling_1, sibling_2, sibling_3] =
            node.siblings.map(|hash| Fq::from(hash).to_bytes().to_vec());
        let mut path = VecDeque::from(node.child);
        path.push_front(pb::MerklePathChunk {
            sibling_1,
            sibling_2,
            sibling_3,
        });
        path.into()
    }
}

// To create `Node<Child>`, we have a recursive impl for `VecDeque` which we delegate to, then
// finally turn into a `Vec` at the end.
impl<Child> TryFrom<VecDeque<pb::MerklePathChunk>> for Node<Child>
where
    Child: TryFrom<VecDeque<pb::MerklePathChunk>, Error = PathDecodeError>,
{
    type Error = PathDecodeError;

    fn try_from(mut queue: VecDeque<pb::MerklePathChunk>) -> Result<Node<Child>, Self::Error> {
        if let Some(pb::MerklePathChunk {
            sibling_1,
            sibling_2,
            sibling_3,
        }) = queue.pop_front()
        {
            let child = Child::try_from(queue)?;
            Ok(Node {
                siblings: [
                    Hash::new(
                        Fq::from_bytes_checked(&sibling_1.try_into().map_err(|_| PathDecodeError)?)
                            .map_err(|_| PathDecodeError)?,
                    ),
                    Hash::new(
                        Fq::from_bytes_checked(&sibling_2.try_into().map_err(|_| PathDecodeError)?)
                            .map_err(|_| PathDecodeError)?,
                    ),
                    Hash::new(
                        Fq::from_bytes_checked(&sibling_3.try_into().map_err(|_| PathDecodeError)?)
                            .map_err(|_| PathDecodeError)?,
                    ),
                ],
                child,
            })
        } else {
            Err(PathDecodeError)
        }
    }
}

impl<Child> TryFrom<Vec<pb::MerklePathChunk>> for Node<Child>
where
    Node<Child>: TryFrom<VecDeque<pb::MerklePathChunk>>,
{
    type Error = <Node<Child> as TryFrom<VecDeque<pb::MerklePathChunk>>>::Error;

    fn try_from(queue: Vec<pb::MerklePathChunk>) -> Result<Node<Child>, Self::Error> {
        <Node<Child>>::try_from(VecDeque::from(queue))
    }
}
