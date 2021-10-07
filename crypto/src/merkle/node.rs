use decaf377::Fq;

use crate::error::MerkleTreeError;
use crate::merkle::constants::{ARITY, MERKLE_PADDING};
use crate::merkle::hash::merkle_hash;
use crate::note;

/// A `Node` in a Merkle `Tree`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    // The value of this node in the merkle tree.
    pub hash_value: Fq,
}

impl Node {
    /// Create a `Node` from the four hash values of its children.
    pub(crate) fn new_parent(layer: usize, children: [Fq; ARITY]) -> Result<Self, MerkleTreeError> {
        Ok(Node {
            hash_value: merkle_hash(
                layer as u32,
                (children[0], children[1], children[2], children[3]),
            ),
        })
    }

    /// Create a new leaf node from a `NoteCommitment` at layer `MERKLE_DEPTH`.
    pub(crate) fn new_leaf(item: note::Commitment) -> Self {
        // Note: This does not hash the items again to save computation
        // as these items are already valid hashes.
        Node { hash_value: item.0 }
    }

    /// Create a new empty `Node` at the given layer.
    pub(crate) fn new_node_at_layer(layer: u32) -> Self {
        let hash_value = merkle_hash(
            layer,
            (
                *MERKLE_PADDING,
                *MERKLE_PADDING,
                *MERKLE_PADDING,
                *MERKLE_PADDING,
            ),
        );

        Node { hash_value }
    }
}

impl std::cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash_value.partial_cmp(&other.hash_value)
    }
}

impl std::cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash_value.cmp(&other.hash_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::One;

    #[test]
    fn test_ordering() {
        let n1 = Node {
            hash_value: Fq::one(),
        };
        let n2 = Node {
            hash_value: Fq::one() + Fq::one(),
        };

        assert_eq!(n2 > n1, true);
    }
}
