use std::collections::VecDeque;

use hash_hasher::HashedMap;

use crate::{
    epoch::{Position, Proof},
    internal::{active::Insert, hash::Hash},
    Commitment, Witness,
};

use super::{block, tree::Tree, InsertError, Tier, TIER_CAPACITY};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    pub epoch: Tier<Tier<Commitment>>,
}

impl Builder {
    pub fn insert(
        &mut self,
        witness: Witness,
        commitment: Commitment,
    ) -> Result<Position, InsertError> {
        let insert = match witness {
            Witness::Keep => Insert::Keep(commitment),
            Witness::Forget => Insert::Hash(Hash::of(commitment)),
        };

        // Fail if epoch is full
        if self.epoch.len() >= TIER_CAPACITY {
            return Err(InsertError::EpochFull);
        }

        // Ensure epoch is not empty
        if self.epoch.is_empty() {
            self.epoch.push_back(Insert::Keep(VecDeque::new()))
        }

        match self
            .epoch
            .back_mut()
            .expect("a new block is added if tiers are empty")
        {
            Insert::Hash(_) => Err(InsertError::BlockForgotten),
            Insert::Keep(block) => {
                // Fail if block is full
                if block.len() >= TIER_CAPACITY {
                    return Err(InsertError::BlockFull);
                }

                // Insert the item into the block
                block.push_back(insert);
                // Calculate the item's position
                let position = (block.len() as u32 - 1) | ((self.epoch.len() as u32 - 1) << 16);
                // Return the position
                Ok(position.into())
            }
        }
    }

    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;
        for insert_block in self.epoch.iter_mut() {
            if let Insert::Keep(block) = insert_block {
                for insert_commitment in block.iter_mut() {
                    if let Insert::Keep(c) = insert_commitment {
                        if commitment == *c {
                            *insert_commitment = Insert::Hash(Hash::of(commitment));
                            forgotten = true;
                        }
                    }
                }
            }
        }
        forgotten
    }

    pub fn insert_block(&mut self, block: block::Builder) -> Result<(), InsertError> {
        if self.epoch.len() < TIER_CAPACITY {
            self.epoch.push_back(Insert::Keep(block.block));
            Ok(())
        } else {
            Err(InsertError::EpochFull)
        }
    }

    pub fn insert_block_root(&mut self, block_root: Hash) -> Result<(), InsertError> {
        if self.epoch.len() < TIER_CAPACITY {
            self.epoch.push_back(Insert::Hash(block_root));
            Ok(())
        } else {
            Err(InsertError::EpochFull)
        }
    }

    pub fn build(self) -> Epoch {
        let tree = Tree::from_epoch(self.epoch);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, (position as u32).into());
        });
        Epoch { index, tree }
    }
}

pub struct Epoch {
    index: HashedMap<Commitment, Position>,
    tree: Tree,
}

impl Epoch {
    pub fn root(&self) -> Hash {
        self.tree.root()
    }

    pub fn current_block_root(&self) -> Option<Hash> {
        let mut tree = &self.tree;
        for _ in 0..8 {
            if let Tree::Node { children, .. } = tree {
                tree = children.last()?;
            } else {
                return None;
            }
        }
        Some(tree.root())
    }

    pub fn position(&self) -> Position {
        (self.tree.position(16) as u32).into()
    }

    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        if let Tree::Node { ref children, hash } = self.tree {
            hash == Hash::default() && children.is_empty()
        } else {
            false
        }
    }

    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(u32::from(position) as u64);
        Some(Proof::new(commitment, position, auth_path))
    }
}
