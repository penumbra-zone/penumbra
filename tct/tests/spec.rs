use std::collections::VecDeque;

use hash_hasher::HashedMap;
use penumbra_tct::{
    internal::{active::Insert, hash::Hash, path::WhichWay},
    Commitment, Position, Proof,
};

type Tier<T> = VecDeque<Insert<T>>;

pub struct Builder {
    tiers: Tier<Tier<Tier<Commitment>>>,
}

pub struct Eternity {
    index: HashedMap<Commitment, Position>,
    tree: Tree,
}

impl Eternity {
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(position.into());
        Some(Proof::new(commitment, position, auth_path))
    }
}

impl Builder {
    pub fn build(self) -> Eternity {
        let tree = Tree::from_tiers(self.tiers);
        let index = tree.index();
        Eternity { index, tree }
    }
}

pub enum Tree {
    Node {
        hash: Hash,
        children: Vec<Tree>,
    },
    Leaf {
        hash: Hash,
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

    /// Construct an entire tree from three nested tiers.
    fn from_tiers(tiers: Tier<Tier<Tier<Commitment>>>) -> Tree {
        use Tree::*;

        let forest = tiers
            .into_iter()
            .map(|insert_epoch| match insert_epoch {
                Insert::Hash(hash) => Node {
                    hash,
                    children: vec![],
                },
                Insert::Keep(epoch) => {
                    let forest = epoch
                        .into_iter()
                        .map(|insert_block| match insert_block {
                            Insert::Hash(hash) => Node {
                                hash,
                                children: vec![],
                            },
                            Insert::Keep(block) => {
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
                        })
                        .collect();
                    Tree::from_tier(8, forest)
                }
            })
            .collect();
        Tree::from_tier(16, forest)
    }

    /// Given a forest of trees of the next smaller tier, assemble them into a single, larger tier.
    ///
    /// # Panics
    ///
    /// If the size of the forest is greater than 2^8.
    fn from_tier(leaf_height: u8, mut forest: VecDeque<Tree>) -> Tree {
        use Tree::*;

        // An empty tier should result in a node with the default hash
        if forest.is_empty() {
            return Node {
                hash: Hash::default(),
                children: vec![],
            };
        }

        for height in leaf_height + 1..=leaf_height + 8 {
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
            assert!(forest.is_empty());
            tree
        } else {
            unreachable!("forest is empty");
        }
    }

    /// Construct an index for all the commitments in the tree.
    fn index(&self) -> HashedMap<Commitment, Position> {
        // Recursive function to build the hash map
        fn index_onto(tree: &Tree, index_here: u64, index: &mut HashedMap<Commitment, Position>) {
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
                    index.insert(*commitment, index_here.into());
                }
                Node { children, .. } => {
                    // Index each child of the node
                    for (i, child) in children.iter().enumerate() {
                        // Index of child node is (4 * index of parent) + {0,1,2,3}
                        let index_here = (index_here << 2) | (i as u64);
                        // Recursively index the child
                        index_onto(child, index_here, index);
                    }
                }
            }
        }

        // Call the recursive hash map builder on an empty starting map, then return the result
        let mut index = HashedMap::default();
        index_onto(self, 0, &mut index);
        index
    }

    /// Get the auth path for a given position, assuming that the tree is of the given height.
    ///
    /// # Panics
    ///
    /// If the index does not correspond to a leaf in the tree, or if the height is greater than
    /// `u8::MAX as usize` or if the height does not match the actual height of the tree.
    fn witness<const HEIGHT: usize>(&self, index: u64) -> [[Hash; 3]; HEIGHT] {
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
