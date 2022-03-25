use std::collections::VecDeque;

use crate::{
    internal::{active::Insert, hash::Hash, path::WhichWay},
    Commitment,
};

use super::Tier;

/// A dense, non-incrememntal merkle tree with commitments at its leaves.
pub enum Tree {
    /// An internal node, with a hash.
    Node {
        /// The hash of this node.
        hash: Hash,
        /// The children of this node (invariant: there are never more than 4).
        children: Vec<Tree>,
    },
    /// A leaf node, with a hash.
    Leaf {
        /// The hash of this leaf.
        hash: Hash,
        /// The commitment witnessed by this leaf, if it was not forgotten.
        commitment: Option<Commitment>,
    },
}

impl Tree {
    /// Get the root hash of this tree.
    pub fn root(&self) -> Hash {
        match self {
            Tree::Node { hash, .. } => *hash,
            Tree::Leaf { hash, .. } => *hash,
        }
    }

    /// Get the position of the rightmost leaf of the tree.
    ///
    /// # Panics
    ///
    /// If the height given is not exactly the height of the tree.
    pub(super) fn position(&self, height: u8) -> u64 {
        match self {
            Tree::Leaf { .. } => {
                if height == 0 {
                    0
                } else {
                    panic!("incorrect height given to Tree::position")
                }
            }
            Tree::Node { children, .. } => {
                if let Some(last) = children.last() {
                    (children.len() as u64) * 4u64.pow(height as u32 - 1)
                        + last.position(height - 1)
                } else {
                    4u64.pow(height as u32)
                }
            }
        }
    }

    /// Construct an entire eternity tree from three nested tiers.
    pub(super) fn from_eternity(eternity: Tier<Tier<Tier<Commitment>>>) -> Tree {
        use Tree::*;

        let forest = eternity
            .into_iter()
            .map(|insert_epoch| match insert_epoch {
                Insert::Keep(epoch) => Tree::from_epoch(epoch),
                Insert::Hash(hash) => Node {
                    hash,
                    children: vec![],
                },
            })
            .collect();
        Tree::from_tier(16, forest)
    }

    /// Construct an entire epoch tree from two nested tiers.
    pub(super) fn from_epoch(epoch: Tier<Tier<Commitment>>) -> Tree {
        use Tree::*;

        let forest = epoch
            .into_iter()
            .map(|insert_block| match insert_block {
                Insert::Keep(block) => Tree::from_block(block),
                Insert::Hash(hash) => Node {
                    hash,
                    children: vec![],
                },
            })
            .collect();
        Tree::from_tier(8, forest)
    }

    /// Construct an entire block tree from one tier.
    pub(super) fn from_block(block: Tier<Commitment>) -> Tree {
        use Tree::*;

        let forest = block
            .into_iter()
            .map(|insert_commitment| match insert_commitment {
                Insert::Hash(hash) => Leaf {
                    hash,
                    commitment: None,
                },
                Insert::Keep(commitment) => Leaf {
                    hash: Hash::of(commitment),
                    commitment: Some(commitment),
                },
            })
            .collect();
        Tree::from_tier(0, forest)
    }

    /// Given a forest of trees of the next smaller tier, assemble them into a single, larger tier.
    ///
    /// # Panics
    ///
    /// If the size of the forest is greater than 4^8.
    fn from_tier(base_height: u8, mut forest: VecDeque<Tree>) -> Tree {
        use Tree::*;

        // An empty tier should result in a node with the default hash
        if forest.is_empty() {
            return Node {
                hash: Hash::default(),
                children: vec![],
            };
        }

        for height in base_height + 1..=base_height + 8 {
            let mut new_forest = VecDeque::with_capacity(
                // Number of subtrees in the new forest is ceiling(len / 4)
                forest.len() / 4 + if forest.len() % 4 == 0 { 0 } else { 1 },
            );

            // Iterate from the front to the back of the forest, building up a new forest also from
            // the front to the back (this is why we need a VecDeque: we use both ends!)
            while !forest.is_empty() {
                // Collect the children and child hashes for this node by grabbing up to 4 trees
                // from the *front* of the forest (defaulting hashes if there are less than 4)
                let mut child_hashes = [Hash::default(); 4];
                let mut children = Vec::with_capacity(4);

                // Because `child_hashes` is of length 4, this will grab at most 4 elements from the
                // front of the forest
                for child_hash in child_hashes.as_mut_slice() {
                    if let Some(tree) = forest.pop_front() {
                        *child_hash = tree.root();
                        children.push(tree);
                    } else {
                        // The forest is empty, so break out of the loop (this is just a slight
                        // performance optimization, because if we finished the loop, we would just
                        // keep hitting this else clause over and over)
                        break;
                    }
                }

                // Compute the root hash of this node
                let [a, b, c, d] = child_hashes;
                let hash = Hash::node(height, a, b, c, d);

                // Push the node onto the *back* of the new forest
                new_forest.push_back(Node { hash, children });
            }
            forest = new_forest;
        }

        if let Some(tree) = forest.pop_front() {
            assert!(forest.is_empty(), "maximum size for tier exceeded");
            tree
        } else {
            unreachable!("forest is empty");
        }
    }

    /// Construct an index for all the commitments in the tree using the given function.
    ///
    /// This is an internally driven iterator that calls the function once for each witnessed leaf
    /// of the tree, in order from left to right.
    pub(super) fn index_with(&self, mut f: impl FnMut(Commitment, u64)) {
        // Recursive function to build the hash map
        fn index_with_at(tree: &Tree, index_here: u64, f: &mut impl FnMut(Commitment, u64)) {
            use Tree::*;
            match tree {
                Leaf {
                    commitment: None, ..
                } => {
                    // Commitment was not witnessed, so we can't index it
                }
                Leaf {
                    commitment: Some(commitment),
                    ..
                } => {
                    // Commitment was witnessed, so index it
                    f(*commitment, index_here);
                }
                Node { children, .. } => {
                    // Index each child of the node
                    for (i, child) in children.iter().enumerate() {
                        // Index of child node is (4 * index of parent) + {0,1,2,3}
                        let index_here = (index_here << 2) | (i as u64);
                        // Recursively index the child
                        index_with_at(child, index_here, f);
                    }
                }
            }
        }

        // Call the recursive hash map builder on an empty starting map, then return the result
        index_with_at(self, 0, &mut f);
    }

    /// Get the auth path for a given position, assuming that the tree is of the given height.
    ///
    /// # Panics
    ///
    /// If the index does not correspond to a leaf in the tree, or if the height is greater than
    /// `u8::MAX as usize` or if the height does not match the actual height of the tree.
    pub(super) fn witness<const HEIGHT: usize>(&self, index: u64) -> [[Hash; 3]; HEIGHT] {
        // Recursive function to build the auth path
        fn witness_onto(tree: &Tree, height: u8, index: u64, auth_path: &mut Vec<[Hash; 3]>) {
            if let Tree::Node { children, .. } = tree {
                // Collect the children into an array of references of exactly size 4
                let mut child_refs: [Option<&Tree>; 4] = [None; 4];
                for (child_ref, child) in child_refs.as_mut_slice().iter_mut().zip(children.iter())
                {
                    *child_ref = Some(child);
                }

                // Determine which way down to go, and what the next index should be
                let (which_way, index) = WhichWay::at(height, index);

                // Pick the child to witness and its siblings
                let (child, siblings) = which_way.pick(child_refs);

                // If the child picked wasn't represented, panic (we expect the index to be valid)
                let child = if let Some(child) = child {
                    child
                } else {
                    panic!("unrepresented index");
                };

                // Calculate the hash of each sibling, using the default hash for those which
                // weren't present
                let siblings =
                    siblings.map(|sibling| sibling.map(|tree| tree.root()).unwrap_or_default());

                // Push the sibling hashes onto the front of the auth path, then recurse into the
                // child to push the rest of the path onto the back of the auth path
                auth_path.push(siblings);
                witness_onto(child, height - 1, index, auth_path);
            } else if height != 0 {
                panic!("leaf at non-zero height");
            }
        }

        // Call the recursive auth path builder on an empty vec, then return the result
        let mut auth_path = Vec::with_capacity(HEIGHT as usize);
        let height = HEIGHT
            .try_into()
            .expect("height must be less than `u8::MAX as usize`");
        witness_onto(self, height, index, &mut auth_path);
        auth_path
            .try_into()
            .expect("height returned by witness_onto will always match requested height")
    }
}
