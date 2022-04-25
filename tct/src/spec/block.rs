//! A specification of the behavior of [`Block`](crate::Block).

use hash_hasher::HashedMap;

use crate::{
    block::{Position, Proof},
    internal::{frontier::Insert, hash::Hash},
    Commitment, Witness,
};

use super::{tree::Tree, InsertError, Tier, TIER_CAPACITY};

/// A builder for a [`Block`]: a sequence of [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    /// The inner tiers of the builder.
    pub block: Tier<Commitment>,
}

impl Builder {
    /// Insert a new [`Commitment`] into the [`block::Builder`](Builder), returning its [`Position`] if
    /// successful.
    ///
    /// See [`crate::Block::insert`].
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
        let position = self.position();
        // Return the position
        Ok(position.into())
    }

    /// Forget the witness for a given [`Commitment`], returning `true` if it was previously witnessed.
    ///
    /// See [`crate::Block::forget`].
    ///
    /// This operation requires a linear scan through the entire builder's contents, and as such
    /// takes time linear in the size of the builder, as opposed to its counterpart,
    ///  [`crate::Block::forget`], which is constant time.
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

    /// Calculate the position of the next insertion into this epoch.
    pub fn position(&self) -> Position {
        let position = self.block.len() as u16 - 1;
        position.into()
    }

    /// Build an immutable, dense commitment tree, finalizing this builder.
    ///
    /// This is not a mirror of any method on [`crate::Block`], because the main crate interface
    /// is incremental, not split into a builder phase and a finalized phase.
    pub fn build(self) -> Block {
        // Calculate position
        let position = (self.block.len() as u16).into();

        let tree = Tree::from_block(self.block);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, (position as u16).into());
        });
        Block {
            position,
            index,
            tree,
        }
    }
}

/// An immutable, dense, indexed commitment tree.
///
/// This supports all the immutable methods of [`crate::Block`].
pub struct Block {
    index: HashedMap<Commitment, Position>,
    position: Position,
    tree: Tree,
}

impl Block {
    /// Get the root hash of this [`Block`].
    ///
    /// See [`crate::Block::root`].
    pub fn root(&self) -> crate::block::Root {
        crate::block::Root(self.tree.root())
    }

    /// Get a [`Proof`] of inclusion for the given [`Commitment`], if it was witnessed.
    ///
    /// See [`crate::Block::witness`].
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(u16::from(position) as u64);
        Some(Proof::new(commitment, position, auth_path))
    }

    /// Get the [`Position`] at which the next [`Commitment`] would be inserted.
    ///
    /// See [`crate::Block::position`].
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get the position of the given [`Commitment`], if it is witnessed.
    ///
    /// See [`crate::Block::position_of`].
    pub fn position_of(&self, commitment: Commitment) -> Option<Position> {
        self.index.get(&commitment).map(|p| *p)
    }

    /// Get the number of [`Commitment`]s witnessed in this [`Block`].
    ///
    /// See [`crate::Block::witnessed_count`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Block`] is empty.
    ///
    /// See [`crate::Block::is_empty`].
    pub fn is_empty(&self) -> bool {
        if let Tree::Node { ref children, hash } = self.tree {
            hash == Hash::default() && children.is_empty()
        } else {
            false
        }
    }
}
