use std::collections::HashMap;

use decaf377::Fq;

use crate::merkle::constants::{ARITY, MERKLE_DEPTH};
use crate::merkle::error::Error;
use crate::merkle::node::Node;
use crate::note;

// TODO serialize/deserialize

/// A 4-arity Merkle tree for note commitments.
///
/// The root node is at layer 0.
pub struct Tree {
    // Datastore of nodes. We index into this hashmap
    // using a single integer that ranges from 0 to the total number of
    // internal nodes.
    //
    // The indexing is breath-wise, i.e. the root node
    // is at 0, the leftmost child node of the root is at 1,
    // and so on.
    //
    // A hash map is used such that only internal nodes that need
    // to exist are populated.
    internals: HashMap<usize, Node>,

    // The index of the next leaf node to be inserted.
    leaf_insert_idx: usize,
}

impl Tree {
    /// Creates an entirely new tree from a list of note commitments.
    pub fn new(items: Vec<note::Commitment>) -> Result<Self, Error> {
        let mut leaves: Vec<Node> = Vec::new();
        for item in &items {
            leaves.push(Node::new_leaf(*item));
        }

        if leaves.len() > Tree::number_nodes_at_layer(MERKLE_DEPTH) {
            return Err(Error::TooLarge);
        }

        let mut internals = HashMap::new();

        // Sorts nodes by their hash value. We can only do this for new trees.
        leaves.sort();

        let mut idx = Tree::layer_start_idx(MERKLE_DEPTH);
        for leaf in &leaves {
            internals.insert(idx, *leaf);
            idx += 1;
        }

        let mut tree = Tree {
            internals,
            leaf_insert_idx: Tree::layer_start_idx(MERKLE_DEPTH) + leaves.len(),
        };
        if leaves.len() > 0 {
            tree.generate();
        } else {
            // Create only a root node and return.
            tree.create_parent(0, 0);
        }

        Ok(tree)
    }

    /// Get the Merkle root.
    pub fn root_hash(&self) -> Fq {
        self.internals
            .get(&0)
            .expect("all trees must have a root node")
            .hash_value
    }

    /// Get the hash value of a node at a given layer and L-R position or fill with default.
    pub fn get_hash_value(&self, layer: usize, lr_position: usize) -> Fq {
        match self.internals.get(&Tree::internal_idx(layer, lr_position)) {
            Some(node) => node.hash_value,
            None => Node::new_node_at_layer(layer as u32).hash_value,
        }
    }

    /// Returns if there is a value at this position in the tree.
    pub fn is_populated(&self, layer: usize, lr_position: usize) -> bool {
        self.internals
            .get(&Tree::internal_idx(layer, lr_position))
            .is_some()
    }

    // Get an array of the children hash values for this node, filling with padding values as needed.
    pub fn get_child_hashes(&self, layer: usize, lr_position: usize) -> Result<[Fq; ARITY], Error> {
        if layer == MERKLE_DEPTH {
            Err(Error::LeafHasNoChild)
        } else {
            Ok([
                self.get_hash_value(layer + 1, lr_position * ARITY + 0),
                self.get_hash_value(layer + 1, lr_position * ARITY + 1),
                self.get_hash_value(layer + 1, lr_position * ARITY + 2),
                self.get_hash_value(layer + 1, lr_position * ARITY + 3),
            ])
        }
    }

    // Get the hash of the parent of the node at a given position.
    pub fn get_parent_hash(&self, layer: usize, lr_position: usize) -> Result<Fq, Error> {
        if layer == 0 {
            Err(Error::RootHasNoParent)
        } else {
            Ok(self.get_hash_value(layer - 1, lr_position / 4))
        }
    }

    /// Incrementally add a new commitment to an existing Merkle tree.
    pub fn add_item(&mut self, item: note::Commitment) {
        self.internals
            .insert(self.leaf_insert_idx, Node::new_leaf(item));
        let lr_position = self.leaf_insert_idx - Tree::layer_start_idx(MERKLE_DEPTH);
        self.leaf_insert_idx += 1;

        // Now starting with this leaf's parent, we traverse the tree,
        // updating parent hashes as we go until we reach the root node.
        for layer in (0..=MERKLE_DEPTH - 1).rev() {
            self.create_parent(layer, lr_position / ARITY);
        }
    }

    /// The number of nodes at this layer.
    fn number_nodes_at_layer(n: usize) -> usize {
        ARITY.pow(n as u32)
    }

    /// Index of a node at a given layer and L-R position.
    fn internal_idx(layer: usize, lr_position: usize) -> usize {
        let count: usize = (0..layer)
            .map(|x| Tree::number_nodes_at_layer(x))
            .sum::<usize>();
        count + lr_position
    }

    /// Layer starts at this position in the internal datastore.
    fn layer_start_idx(layer: usize) -> usize {
        Tree::internal_idx(layer, 0)
    }

    /// Called when we first generate the internal nodes of the tree (non-incremental).
    fn generate(&mut self) {
        for i in (0..=MERKLE_DEPTH - 1).rev() {
            self.add_layer(i);
        }
    }

    /// Create a parent node at this position in the tree from its children.
    fn create_parent(&mut self, layer: usize, lr_position: usize) {
        let idx = Tree::internal_idx(layer, lr_position);
        let children = self
            .get_child_hashes(layer, lr_position)
            .expect("got children successfully");
        self.internals.insert(
            idx,
            Node::new_parent(layer, children).expect("node created successfully"),
        );
    }

    /// Add a layer of parent nodes to the tree.
    fn add_layer(&mut self, layer: usize) {
        // Stride by `ARITY` through the layer below, only adding a parent hash when needed.
        for lr_position in (0..Tree::number_nodes_at_layer(layer + 1)).step_by(ARITY) {
            // Only if there is a node do we create parent to save space in the internal datastore.
            if self.is_populated(layer + 1, lr_position) {
                self.create_parent(layer, lr_position / ARITY);
            } else {
                // Once we hit a non-populated node, we can just skip to the next layer, since
                // the tree gets filled from L to R.
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use crate::addresses::PaymentAddress;
    use crate::asset;
    use crate::keys::{Diversifier, IncomingViewingKey, SpendKey};
    use crate::merkle::constants::MERKLE_DEPTH;
    use crate::merkle::hash::merkle_hash;
    use crate::value::Value;

    fn note_commitment_factory(n: usize) -> Vec<note::Commitment> {
        let mut rng = OsRng;
        let pen_trace = b"pen";
        let pen_id = asset::Id::from(&pen_trace[..]);
        let value = Value {
            amount: 10,
            asset_id: pen_id,
        };

        let diversifier = Diversifier::generate(&mut rng);
        let expanded_sk = SpendKey::generate(&mut rng);
        let proof_auth_key = expanded_sk.proof_authorization_key();
        let fvk = expanded_sk.full_viewing_key();
        let ivk = IncomingViewingKey::derive(&proof_auth_key.ak, &fvk.nk);
        let pk_d = ivk.derive_transmission_key(&diversifier);
        let dest = PaymentAddress::new(diversifier, pk_d);

        let mut cms = Vec::<note::Commitment>::new();
        for _i in 0..n {
            let note_blinding = Fq::rand(&mut rng);
            let cm = note::Commitment::new(&dest, &value, &note_blinding);
            cms.push(cm);
        }
        cms
    }

    #[test]
    fn construct_empty_merkle_tree() {
        let empty_vec = Vec::new();

        let tree = Tree::new(empty_vec).expect("empty tree created");

        assert_eq!(
            format!("{}", tree.root_hash()),
            "51236103744426496202047844657403854516464413940747120247768585921342213104"
        );
    }

    #[test]
    fn construct_note_commitment_merkle_tree_no_incremental() {
        let commitments = note_commitment_factory(4);

        let tree = Tree::new(commitments).expect("can create test tree");

        // Root node and four leaves should exist.
        assert!(tree.is_populated(0, 0) == true);
        for i in 0..4 {
            assert!(tree.is_populated(MERKLE_DEPTH, i) == true);
        }
        // Notes to the right of the 4th leaf should not exist.
        for i in 4..100 {
            assert!(tree.is_populated(MERKLE_DEPTH, i) == false);
        }

        // A node should be the hash of its four children.
        let children = tree
            .get_child_hashes(0, 0)
            .expect("can get root node children");
        let expected_root_hash =
            merkle_hash(0, (children[0], children[1], children[2], children[3]));

        assert_eq!(expected_root_hash, tree.root_hash());
    }

    #[test]
    fn construct_note_commitment_merkle_tree_incremental() {
        let commitments = note_commitment_factory(3);
        let mut tree = Tree::new(commitments).expect("can create test tree");

        // Assert tree is in expected state before updating.
        assert!(tree.is_populated(0, 0) == true);
        for i in 0..3 {
            assert!(tree.is_populated(MERKLE_DEPTH, i) == true);
        }
        assert!(tree.is_populated(MERKLE_DEPTH, 3) == false);
        let merkle_root_before = tree.root_hash();

        // Now create another note commitment, and add it to the merkle tree.
        let commitments = note_commitment_factory(1);
        tree.add_item(commitments[0]);

        assert!(tree.is_populated(0, 0) == true);
        for i in 0..3 {
            assert!(tree.is_populated(MERKLE_DEPTH, i) == true);
        }
        // Now a new leaf node should exist.
        assert!(tree.is_populated(MERKLE_DEPTH, 3) == true);

        // Merkle root should change when a new commitment is added.
        assert_ne!(merkle_root_before, tree.root_hash());
    }

    #[test]
    fn test_num_nodes_helper_methods() {
        assert_eq!(Tree::number_nodes_at_layer(0), 1);
        assert_eq!(Tree::number_nodes_at_layer(1), 4);
        assert_eq!(Tree::number_nodes_at_layer(2), 16);
        assert_eq!(Tree::number_nodes_at_layer(3), 64);
        assert_eq!(Tree::number_nodes_at_layer(4), 256);
    }

    #[test]
    fn test_index_helper_methods() {
        assert_eq!(Tree::layer_start_idx(0), 0);
        assert_eq!(Tree::layer_start_idx(1), 1);
        assert_eq!(Tree::layer_start_idx(2), 5);
        assert_eq!(Tree::layer_start_idx(3), 21);
        assert_eq!(Tree::layer_start_idx(4), 85);
    }
}
