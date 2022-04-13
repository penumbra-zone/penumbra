//! A specification of the behavior of [`Epoch`](crate::Epoch).

use std::collections::VecDeque;

use hash_hasher::HashedMap;

use crate::{
    epoch::{Position, Proof},
    internal::{active::Insert, hash::Hash, index},
    Commitment, Witness,
};

use super::{block, tree::Tree, InsertError, Tier, TIER_CAPACITY};

/// A builder for an [`Epoch`]: a sequence of blocks, each of which is a sequence of [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    /// The inner tiers of the builder.
    pub epoch: Tier<Tier<Commitment>>,
}

impl Builder {
    /// Insert a new [`Commitment`] into the [`epoch::Builder`](Builder), returning its [`Position`] if
    /// successful.
    ///
    /// See [`crate::Epoch::insert`].
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

        // Calculate position
        let position = self.position();

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

                // Return the position
                Ok(position)
            }
        }
    }

    /// Forget the witness for a given [`Commitment`], returning `true` if it was previously witnessed.
    ///
    /// See [`crate::Epoch::forget`].
    ///
    /// This operation requires a linear scan through the entire builder's contents, and as such
    /// takes time linear in the size of the builder, as opposed to its counterpart,
    ///  [`crate::Epoch::forget`], which is constant time.
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

    /// Calculate the position of the next insertion into this epoch.
    fn position(&self) -> Position {
        let (block, commitment) = if self.epoch.is_empty() {
            (0.into(), 0.into())
        } else {
            let commitment = match self.epoch.back().unwrap() {
                Insert::Hash(_) => index::Commitment::MAX,
                Insert::Keep(block) => (block.len() as u16).into(),
            };
            (((self.epoch.len() - 1) as u16).into(), commitment)
        };

        Position::from(u32::from(crate::internal::index::within::Epoch {
            block,
            commitment,
        }))
    }

    /// Insert a block builder's contents as a new block in this [`epoch::Builder`](Builder).
    ///
    /// See [`crate::Epoch::insert_block`].
    pub fn insert_block(&mut self, block: block::Builder) -> Result<(), InsertError> {
        if self.epoch.len() < TIER_CAPACITY {
            self.epoch.push_back(Insert::Keep(block.block));
            Ok(())
        } else {
            Err(InsertError::EpochFull)
        }
    }

    /// Insert a block root as a new block root in this [`epoch::Builder`](Builder).
    ///
    /// See [`crate::Epoch::insert_block_root`].
    pub fn insert_block_root(
        &mut self,
        crate::block::Root(block_root): crate::block::Root,
    ) -> Result<(), InsertError> {
        if self.epoch.len() < TIER_CAPACITY {
            self.epoch.push_back(Insert::Hash(block_root));
            Ok(())
        } else {
            Err(InsertError::EpochFull)
        }
    }

    /// Build an immutable, dense commitment tree, finalizing this builder.
    ///
    /// This is not a mirror of any method on [`crate::Epoch`], because the main crate interface
    /// is incremental, not split into a builder phase and a finalized phase.
    pub fn build(self) -> Epoch {
        let position = self.position();
        let tree = Tree::from_epoch(self.epoch);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, (position as u32).into());
        });
        Epoch {
            position,
            index,
            tree,
        }
    }
}

/// An immutable, dense, indexed commitment tree.
///
/// This supports all the immutable methods of [`crate::Epoch`].
pub struct Epoch {
    index: HashedMap<Commitment, Position>,
    position: Position,
    tree: Tree,
}

impl Epoch {
    /// Get the root hash of this [`Epoch`].
    ///
    /// See [`crate::Epoch::root`].
    pub fn root(&self) -> crate::epoch::Root {
        crate::epoch::Root(self.tree.root())
    }

    /// Get a [`Proof`] of inclusion for the given [`Commitment`], if it was witnessed.
    ///
    /// See [`crate::Epoch::witness`].
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(u32::from(position) as u64);
        Some(Proof::new(commitment, position, auth_path))
    }

    /// Get the block root of the current block of this [`Epoch`], if any.
    ///
    /// See [`crate::Epoch::current_block_root`].
    pub fn current_block_root(&self) -> Option<crate::block::Root> {
        let mut tree = &self.tree;
        for _ in 0..8 {
            if let Tree::Node { children, .. } = tree {
                tree = children.last()?;
            } else {
                return None;
            }
        }
        Some(crate::block::Root(tree.root()))
    }

    /// Get the [`Position`] at which the next [`Commitment`] would be inserted.
    ///
    /// See [`crate::Epoch::position`].
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get the number of [`Commitment`]s witnessed in this [`Epoch`].
    ///
    /// See [`crate::Epoch::witnessed_count`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Epoch`] is empty.
    ///
    /// See [`crate::Epoch::is_empty`].
    pub fn is_empty(&self) -> bool {
        if let Tree::Node { ref children, hash } = self.tree {
            hash == Hash::default() && children.is_empty()
        } else {
            false
        }
    }
}
