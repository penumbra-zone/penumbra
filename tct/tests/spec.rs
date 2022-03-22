use penumbra_tct::internal::hash::Hash;

pub enum Tree {
    Node {
        hash: Hash,
        children: Box<[Option<Tree>; 4]>,
    },
    Leaf {
        hash: Hash,
    },
    Subtree {
        subtree: Box<Tree>,
    },
}

impl Tree {
    pub fn hash(&self) -> Hash {
        match self {
            Tree::Node { hash, .. } => *hash,
            Tree::Leaf { hash, .. } => *hash,
            Tree::Subtree { subtree, .. } => subtree.hash(),
        }
    }

    pub fn from_tiers(tiers: Vec<Vec<Vec<Hash>>>) -> Tree {
        use Tree::*;

        let forest = tiers
            .into_iter()
            .map(|epoch| {
                let forest = epoch
                    .into_iter()
                    .map(|block| {
                        let forest = block.into_iter().map(|hash| Leaf { hash }).collect();
                        Tree::from_tier(0, forest)
                    })
                    .collect();
                Tree::from_tier(8, forest)
            })
            .collect();
        Tree::from_tier(16, forest)
    }

    fn from_tier(leaf_height: u8, mut forest: Vec<Tree>) -> Tree {
        use Tree::*;

        if forest.is_empty() {
            return Leaf {
                hash: Hash::default(),
            };
        }

        for height in leaf_height + 1..=leaf_height + 8 {
            let mut new_forest =
                Vec::with_capacity(forest.len() / 4 + if forest.len() % 4 == 0 { 0 } else { 1 });
            while !forest.is_empty() {
                let a = forest.pop();
                let b = forest.pop();
                let c = forest.pop();
                let d = forest.pop();
                new_forest.push(Node {
                    hash: Hash::node(
                        height,
                        a.as_ref().map(|t| t.hash()).unwrap_or_else(Hash::default),
                        b.as_ref().map(|t| t.hash()).unwrap_or_else(Hash::default),
                        c.as_ref().map(|t| t.hash()).unwrap_or_else(Hash::default),
                        d.as_ref().map(|t| t.hash()).unwrap_or_else(Hash::default),
                    ),
                    children: Box::new([a, b, c, d]),
                });
            }
            forest = new_forest;
        }

        if let Some(tree) = forest.pop() {
            assert!(forest.is_empty());
            tree
        } else {
            unreachable!("forest is empty");
        }
    }
}
