use std::collections::VecDeque;

use hash_hasher::HashedMap;
use penumbra_tct::{
    internal::{active::Insert, hash::Hash},
    Commitment, Position, Proof, Witness,
};

use crate::InsertError;

use super::{block, epoch, tree::Tree, Tier, TIER_CAPACITY};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    pub eternity: Tier<Tier<Tier<Commitment>>>,
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

        // Fail if eternity is full
        if self.eternity.len() >= TIER_CAPACITY {
            return Err(InsertError::Full);
        }

        // Ensure eternity is not empty
        if self.eternity.is_empty() {
            self.eternity.push_back(Insert::Keep(VecDeque::new()))
        }

        match self
            .eternity
            .back_mut()
            .expect("a new epoch is added if tiers are empty")
        {
            Insert::Hash(_) => Err(InsertError::EpochForgotten),
            Insert::Keep(epoch) => {
                // Fail if epoch is full
                if epoch.len() >= TIER_CAPACITY {
                    return Err(InsertError::EpochFull);
                }

                // Ensure epoch is not empty
                if epoch.is_empty() {
                    epoch.push_back(Insert::Keep(VecDeque::new()));
                }

                match epoch
                    .back_mut()
                    .expect("a new block is added if epoch is empty")
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
                        let position = (block.len() as u64 - 1)
                            | ((epoch.len() as u64 - 1) << 16)
                            | ((self.eternity.len() as u64 - 1) << 32);
                        // Return the position
                        Ok(position.into())
                    }
                }
            }
        }
    }

    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;
        for insert_epoch in self.eternity.iter_mut() {
            if let Insert::Keep(epoch) = insert_epoch {
                for insert_block in epoch.iter_mut() {
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
            }
        }
        forgotten
    }

    pub fn insert_block(&mut self, block: block::Builder) -> Result<(), InsertError> {
        self.insert_block_or_root(Insert::Keep(block))
    }

    pub fn insert_block_root(&mut self, block_root: Hash) -> Result<(), InsertError> {
        self.insert_block_or_root(Insert::Hash(block_root))
    }

    fn insert_block_or_root(&mut self, insert: Insert<block::Builder>) -> Result<(), InsertError> {
        // Fail if eternity is full
        if self.eternity.len() >= TIER_CAPACITY {
            return Err(InsertError::Full);
        }

        // Ensure eternity is not empty
        if self.eternity.is_empty() {
            self.eternity.push_back(Insert::Keep(VecDeque::new()))
        }

        match self
            .eternity
            .back_mut()
            .expect("a new epoch is added if tiers are empty")
        {
            Insert::Hash(_) => Err(InsertError::EpochForgotten),
            Insert::Keep(epoch) => {
                // Fail if epoch is full
                if epoch.len() >= TIER_CAPACITY {
                    return Err(InsertError::EpochFull);
                }

                // Ensure epoch is not empty
                if epoch.is_empty() {
                    epoch.push_back(Insert::Keep(VecDeque::new()));
                }

                // Insert whatever is to be inserted
                if epoch.len() < TIER_CAPACITY {
                    epoch.push_back(insert.map(|block| block.block));
                    Ok(())
                } else {
                    Err(InsertError::EpochFull)
                }
            }
        }
    }

    pub fn insert_epoch(&mut self, epoch: epoch::Builder) -> Result<(), InsertError> {
        if self.eternity.len() < TIER_CAPACITY {
            self.eternity.push_back(Insert::Keep(epoch.epoch));
            Ok(())
        } else {
            Err(InsertError::Full)
        }
    }

    pub fn insert_epoch_root(&mut self, epoch_root: Hash) -> Result<(), InsertError> {
        if self.eternity.len() < TIER_CAPACITY {
            self.eternity.push_back(Insert::Hash(epoch_root));
            Ok(())
        } else {
            Err(InsertError::Full)
        }
    }

    pub fn build(self) -> Eternity {
        let tree = Tree::from_eternity(self.eternity);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, position.into());
        });
        Eternity { index, tree }
    }
}

pub struct Eternity {
    index: HashedMap<Commitment, Position>,
    tree: Tree,
}

impl Eternity {
    pub fn root(&self) -> Hash {
        self.tree.root()
    }

    pub fn current_block_root(&self) -> Option<Hash> {
        let mut tree = &self.tree;
        for _ in 0..16 {
            if let Tree::Node { children, .. } = tree {
                tree = children.last()?;
            } else {
                return None;
            }
        }
        Some(tree.root())
    }

    pub fn current_epoch_root(&self) -> Option<Hash> {
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
        self.tree.position(24).into()
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
        let auth_path = self.tree.witness(position.into());
        Some(Proof::new(commitment, position, auth_path))
    }
}
