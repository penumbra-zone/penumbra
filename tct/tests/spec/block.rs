use hash_hasher::HashedMap;
use penumbra_tct::{
    block::{Position, Proof},
    internal::{active::Insert, hash::Hash},
    Commitment, Witness,
};

use super::{tree::Tree, InsertError, Tier, TIER_CAPACITY};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    pub block: Tier<Commitment>,
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

        // Fail if block is full
        if self.block.len() >= TIER_CAPACITY {
            return Err(InsertError::BlockFull);
        }

        // Insert the item into the block
        self.block.push_back(insert);
        // Calculate the item's position
        let position = self.block.len() as u16 - 1;
        // Return the position
        Ok(position.into())
    }

    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;
        for insert_commitment in self.block.iter_mut() {
            if let Insert::Keep(c) = insert_commitment {
                if commitment == *c {
                    *insert_commitment = Insert::Hash(Hash::of(commitment));
                    forgotten = true;
                }
            }
        }
        forgotten
    }

    pub fn build(self) -> Block {
        let tree = Tree::from_block(self.block);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, (position as u16).into());
        });
        Block { index, tree }
    }
}

pub struct Block {
    index: HashedMap<Commitment, Position>,
    tree: Tree,
}

impl Block {
    pub fn root(&self) -> Hash {
        self.tree.root()
    }

    pub fn position(&self) -> Position {
        (self.tree.position(8) as u16).into()
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
        let auth_path = self.tree.witness(u16::from(position) as u64);
        Some(Proof::new(commitment, position, auth_path))
    }
}
