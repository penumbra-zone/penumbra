//! A specification of the behavior of [`Block`](crate::Block).

use hash_hasher::HashedMap;

use penumbra_tct::block::{Position, Proof};

use crate::*;

/// A builder for a [`Block`]: a sequence of [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    /// The inner tiers of the builder.
    pub block: Vec<Insert<Commitment>>,
}

impl Builder {
    /// Insert a new [`Commitment`] into the [`block::Builder`](Builder), returning its [`Position`] if
    /// successful.
    ///
    /// See [`penumbra_tct::Block::insert`].
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

        // Calculate the item's position
        let position = self.position();

        // Insert the item into the block
        self.block.push(insert);

        // Return the position
        Ok(position)
    }

    /// Forget the witness for a given [`Commitment`], returning `true` if it was previously witnessed.
    ///
    /// See [`penumbra_tct::Block::forget`].
    ///
    /// This operation requires a linear scan through the entire builder's contents, and as such
    /// takes time linear in the size of the builder, as opposed to its counterpart,
    ///  [`penumbra_tct::Block::forget`], which is constant time.
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
    /// This is not a mirror of any method on [`penumbra_tct::Block`], because the main crate
    /// interface is incremental, not split into a builder phase and a finalized phase.
    pub fn build(self) -> Block {
        Block {
            position: self.position(),
            index: index::block(&self.block),
            tree: Tree::from_block(self.block),
        }
    }
}

/// An immutable, dense, indexed commitment tree.
///
/// This supports all the immutable methods of [`penumbra_tct::Block`].
pub struct Block {
    index: HashedMap<Commitment, Position>,
    position: Position,
    tree: Tree,
}

impl Block {
    /// Get the root hash of this [`Block`].
    ///
    /// See [`penumbra_tct::Block::root`].
    pub fn root(&self) -> penumbra_tct::block::Root {
        penumbra_tct::block::Root(self.tree.root())
    }

    /// Get a [`Proof`] of inclusion for the given [`Commitment`], if it was witnessed.
    ///
    /// See [`penumbra_tct::Block::witness`].
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(u16::from(position) as u64);
        Some(Proof::new(commitment, position, auth_path))
    }

    /// Get the [`Position`] at which the next [`Commitment`] would be inserted.
    ///
    /// See [`penumbra_tct::Block::position`].
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get the position of the given [`Commitment`], if it is witnessed.
    ///
    /// See [`penumbra_tct::Block::position_of`].
    pub fn position_of(&self, commitment: Commitment) -> Option<Position> {
        self.index.get(&commitment).copied()
    }

    /// Get the number of [`Commitment`]s witnessed in this [`Block`].
    ///
    /// See [`penumbra_tct::Block::witnessed_count`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Block`] is empty.
    ///
    /// See [`penumbra_tct::Block::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.tree.as_tier().root.is_none()
    }
}
