use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_sdk_tct::StateCommitment;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The default depth of the Quad Merkle Tree.
pub const DEFAULT_DEPTH: u8 = 16;

/// Precomputed zero hashes for each level of the tree (up to depth 16).
/// zero_hashes[0] = hash of empty leaf
/// zero_hashes[i] = hash_4([zero_hashes[i-1]; 4]) for i > 0
pub static ZERO_HASHES: Lazy<Vec<StateCommitment>> = Lazy::new(|| {
    let mut zeros = Vec::with_capacity((DEFAULT_DEPTH + 1) as usize);

    // Level 0: empty leaf hash (zero field element)
    zeros.push(StateCommitment(Fq::from(0u64)));

    // Compute zero hashes for each level up to DEFAULT_DEPTH
    for i in 1..=(DEFAULT_DEPTH as usize) {
        let prev = zeros[i - 1].0;
        // Hash four copies of the previous level's zero hash
        let hash = poseidon377::hash_4(&prev, (prev, prev, prev, prev));
        zeros.push(StateCommitment(hash));
    }

    zeros
});

/// A Quad Merkle Tree (arity 4) using Poseidon377 hashing.
///
/// This tree stores nodes sparsely - only non-zero nodes are stored in the BTreeMap.
/// Missing nodes are implicitly the zero hash for that level.
///
/// Note: BTreeMap is used instead of HashMap to ensure deterministic iteration order,
/// which is required for consistent serialization (especially with bincode).
#[derive(Clone, Debug)]
pub struct QuadTree {
    /// The depth of the tree (number of levels from leaf to root, excluding root).
    depth: u8,
    /// Sparse storage of tree nodes.
    /// Key format: level << 48 | position
    /// - level 0 = leaves
    /// - level depth = root
    nodes: BTreeMap<u64, StateCommitment>,
}

/// Serialization helper to convert StateCommitments to raw bytes.
/// BTreeMap guarantees sorted iteration, making serialization deterministic.
#[derive(Serialize, Deserialize)]
struct QuadTreeSerde {
    depth: u8,
    nodes: Vec<(u64, [u8; 32])>,
}

impl Serialize for QuadTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // BTreeMap iterates in sorted order, so this produces deterministic output
        let nodes: Vec<(u64, [u8; 32])> = self
            .nodes
            .iter()
            .map(|(k, v)| (*k, v.0.to_bytes()))
            .collect();

        let helper = QuadTreeSerde {
            depth: self.depth,
            nodes,
        };
        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QuadTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = QuadTreeSerde::deserialize(deserializer)?;

        let nodes: BTreeMap<u64, StateCommitment> = helper
            .nodes
            .into_iter()
            .map(|(k, bytes)| {
                let fq = Fq::from_bytes_checked(&bytes)
                    .map_err(|_| serde::de::Error::custom("invalid Fq bytes"))?;
                Ok((k, StateCommitment(fq)))
            })
            .collect::<Result<_, D::Error>>()?;

        Ok(QuadTree {
            depth: helper.depth,
            nodes,
        })
    }
}

impl QuadTree {
    /// Create a new empty Quad Merkle Tree with default depth.
    pub fn new() -> Self {
        Self {
            depth: DEFAULT_DEPTH,
            nodes: BTreeMap::new(),
        }
    }

    /// Create a new Quad Merkle Tree with a custom depth.
    pub fn with_depth(depth: u8) -> Self {
        Self {
            depth,
            nodes: BTreeMap::new(),
        }
    }

    /// Compute the storage key for a node at a given level and position.
    #[inline]
    fn node_key(level: u8, position: u64) -> u64 {
        ((level as u64) << 48) | position
    }

    /// Get a node hash at a given level and position.
    /// Returns the zero hash for that level if the node doesn't exist.
    fn get_node(&self, level: u8, position: u64) -> StateCommitment {
        let key = Self::node_key(level, position);
        self.nodes
            .get(&key)
            .copied()
            .unwrap_or_else(|| ZERO_HASHES[level as usize])
    }

    /// Set a node hash at a given level and position.
    fn set_node(&mut self, level: u8, position: u64, hash: StateCommitment) {
        let key = Self::node_key(level, position);
        // Only store non-zero hashes to keep the tree sparse
        if hash.0 != ZERO_HASHES[level as usize].0 {
            self.nodes.insert(key, hash);
        } else {
            self.nodes.remove(&key);
        }
    }

    /// Hash four child nodes to produce the parent hash.
    /// Using child0 as domain separator is a common pattern for Merkle trees.
    fn hash_children(
        child0: StateCommitment,
        child1: StateCommitment,
        child2: StateCommitment,
        child3: StateCommitment,
    ) -> StateCommitment {
        // poseidon377::hash_4 takes (domain_sep, (val1, val2, val3, val4))
        // We use child0 as the domain separator
        let hash = poseidon377::hash_4(&Fq::from(0u64), (child0.0, child1.0, child2.0, child3.0));
        StateCommitment(hash)
    }

    /// Update a leaf at the given position and recalculate the path to the root.
    ///
    /// # Arguments
    /// * `position` - The leaf position (index) to update
    /// * `leaf_hash` - The new hash value for the leaf
    pub fn update(&mut self, position: u64, leaf_hash: StateCommitment) {
        // Check that position is valid for this tree depth
        let max_leaves = 1u64 << (self.depth * 2); // 4^depth = 2^(2*depth)
        if position >= max_leaves {
            panic!(
                "Position {} exceeds maximum leaves {} for depth {}",
                position, max_leaves, self.depth
            );
        }

        // Set the leaf
        self.set_node(0, position, leaf_hash);

        // Recalculate hashes up the tree
        let mut current_position = position;
        for level in 0..self.depth {
            // For a quad tree, each parent has 4 children
            // Parent position = child_position / 4
            let parent_position = current_position / 4;

            // Get the base position of this group of 4 siblings
            let base_position = parent_position * 4;

            // Get all 4 children
            let child0 = self.get_node(level, base_position);
            let child1 = self.get_node(level, base_position + 1);
            let child2 = self.get_node(level, base_position + 2);
            let child3 = self.get_node(level, base_position + 3);

            // Hash them to get parent
            let parent_hash = Self::hash_children(child0, child1, child2, child3);

            // Store the parent
            self.set_node(level + 1, parent_position, parent_hash);

            // Move up to the next level
            current_position = parent_position;
        }
    }

    /// Get the root hash of the tree.
    pub fn root(&self) -> StateCommitment {
        self.get_node(self.depth, 0)
    }

    /// Generate an authentication path for a leaf at the given position.
    ///
    /// Returns a vector of sibling arrays, where each array contains the 3 siblings
    /// at that level (excluding the node on the path itself).
    ///
    /// # Returns
    /// A vector of length `depth`, where each element is `[sibling0, sibling1, sibling2]`.
    /// The order of siblings depends on which child the path goes through:
    /// - If path is child 0: siblings are [1, 2, 3]
    /// - If path is child 1: siblings are [0, 2, 3]
    /// - If path is child 2: siblings are [0, 1, 3]
    /// - If path is child 3: siblings are [0, 1, 2]
    pub fn auth_path(&self, position: u64) -> Vec<[StateCommitment; 3]> {
        let max_leaves = 1u64 << (self.depth * 2);
        if position >= max_leaves {
            panic!(
                "Position {} exceeds maximum leaves {} for depth {}",
                position, max_leaves, self.depth
            );
        }

        let mut path = Vec::with_capacity(self.depth as usize);
        let mut current_position = position;

        for level in 0..self.depth {
            // Determine which child (0-3) we are
            let child_index = (current_position % 4) as usize;

            // Get base position of the 4-sibling group
            let base_position = (current_position / 4) * 4;

            // Get all 4 children
            let children = [
                self.get_node(level, base_position),
                self.get_node(level, base_position + 1),
                self.get_node(level, base_position + 2),
                self.get_node(level, base_position + 3),
            ];

            // Extract the 3 siblings (all except the one at child_index)
            let siblings = match child_index {
                0 => [children[1], children[2], children[3]],
                1 => [children[0], children[2], children[3]],
                2 => [children[0], children[1], children[3]],
                3 => [children[0], children[1], children[2]],
                _ => unreachable!(),
            };

            path.push(siblings);

            // Move to parent position
            current_position /= 4;
        }

        path
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Get the number of stored (non-zero) nodes.
    pub fn num_stored_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get a leaf value at the given position.
    ///
    /// Returns `Some(commitment)` if a leaf exists at this position,
    /// `None` if the position is empty (zero hash).
    pub fn get_leaf(&self, position: u64) -> Option<StateCommitment> {
        let commitment = self.get_node(0, position);
        // Return None if it's the zero hash (empty leaf)
        if commitment.0 == ZERO_HASHES[0].0 {
            None
        } else {
            Some(commitment)
        }
    }

    /// Verify an authentication path for a given leaf.
    ///
    /// # Arguments
    /// * `position` - The leaf position
    /// * `leaf_hash` - The leaf hash to verify
    /// * `auth_path` - The authentication path (3 siblings per level)
    /// * `expected_root` - The expected root hash
    ///
    /// # Returns
    /// `true` if the path is valid, `false` otherwise
    pub fn verify_auth_path(
        position: u64,
        leaf_hash: StateCommitment,
        auth_path: &[[StateCommitment; 3]],
        expected_root: StateCommitment,
        depth: u8,
    ) -> bool {
        let mut current_hash = leaf_hash;
        let mut current_position = position;

        for siblings in auth_path.iter().take(depth as usize) {
            // Determine which child (0-3) we are
            let child_index = (current_position % 4) as usize;

            // Reconstruct the 4 children array
            let children = match child_index {
                0 => [current_hash, siblings[0], siblings[1], siblings[2]],
                1 => [siblings[0], current_hash, siblings[1], siblings[2]],
                2 => [siblings[0], siblings[1], current_hash, siblings[2]],
                3 => [siblings[0], siblings[1], siblings[2], current_hash],
                _ => unreachable!(),
            };

            // Hash to get parent
            current_hash = Self::hash_children(children[0], children[1], children[2], children[3]);

            // Move to parent position
            current_position /= 4;
        }

        current_hash.0 == expected_root.0
    }
}

impl Default for QuadTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_hashes_precomputed() {
        // Ensure zero hashes are precomputed correctly
        assert_eq!(ZERO_HASHES.len(), (DEFAULT_DEPTH + 1) as usize);
        assert_eq!(ZERO_HASHES[0].0, Fq::from(0u64));
    }

    #[test]
    fn test_quad_tree_basic() {
        let mut tree = QuadTree::new();

        // Initially, root should be the zero hash for the depth level
        let initial_root = tree.root();
        assert_eq!(initial_root.0, ZERO_HASHES[DEFAULT_DEPTH as usize].0);

        // Update a leaf
        let leaf_hash = StateCommitment(Fq::from(42u64));
        tree.update(0, leaf_hash);

        // Root should have changed
        let new_root = tree.root();
        assert_ne!(new_root.0, initial_root.0);
    }

    #[test]
    fn test_auth_path_length() {
        let mut tree = QuadTree::new();
        let leaf_hash = StateCommitment(Fq::from(123u64));
        tree.update(5, leaf_hash);

        let path = tree.auth_path(5);
        assert_eq!(path.len(), DEFAULT_DEPTH as usize);
    }

    #[test]
    fn test_verify_auth_path() {
        let mut tree = QuadTree::new();
        let leaf_hash = StateCommitment(Fq::from(999u64));
        let position = 7u64;

        tree.update(position, leaf_hash);
        let root = tree.root();
        let path = tree.auth_path(position);

        // Verification should succeed
        assert!(QuadTree::verify_auth_path(
            position,
            leaf_hash,
            &path,
            root,
            DEFAULT_DEPTH
        ));

        // Verification with wrong leaf should fail
        let wrong_leaf = StateCommitment(Fq::from(111u64));
        assert!(!QuadTree::verify_auth_path(
            position,
            wrong_leaf,
            &path,
            root,
            DEFAULT_DEPTH
        ));
    }

    #[test]
    fn test_multiple_updates() {
        let mut tree = QuadTree::new();

        // Update multiple leaves
        tree.update(0, StateCommitment(Fq::from(1u64)));
        tree.update(1, StateCommitment(Fq::from(2u64)));
        tree.update(2, StateCommitment(Fq::from(3u64)));
        tree.update(3, StateCommitment(Fq::from(4u64)));

        let root = tree.root();

        // Verify each path
        for pos in 0..4u64 {
            let leaf = StateCommitment(Fq::from((pos + 1) as u64));
            let path = tree.auth_path(pos);
            assert!(QuadTree::verify_auth_path(
                pos,
                leaf,
                &path,
                root,
                DEFAULT_DEPTH
            ));
        }
    }

    #[test]
    fn test_sparse_storage() {
        let mut tree = QuadTree::new();

        // Tree should start with 0 stored nodes
        assert_eq!(tree.num_stored_nodes(), 0);

        // Update one leaf
        tree.update(0, StateCommitment(Fq::from(1u64)));

        // Should have stored nodes only on the path from leaf to root
        // For depth 16, that's 17 nodes (leaf + 16 internal levels)
        assert!(tree.num_stored_nodes() <= (DEFAULT_DEPTH + 1) as usize);
    }

    #[test]
    fn test_serialization() {
        let mut tree = QuadTree::new();
        tree.update(0, StateCommitment(Fq::from(1u64)));
        tree.update(10, StateCommitment(Fq::from(2u64)));

        // Serialize
        let serialized = serde_json::to_string(&tree).expect("serialization failed");

        // Deserialize
        let deserialized: QuadTree =
            serde_json::from_str(&serialized).expect("deserialization failed");

        // Should have same root
        assert_eq!(tree.root().0, deserialized.root().0);
        assert_eq!(tree.num_stored_nodes(), deserialized.num_stored_nodes());
    }
}
