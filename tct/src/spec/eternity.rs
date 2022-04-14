//! A specification of the behavior of [`Eternity`](crate::Eternity).

use std::{collections::VecDeque, marker::PhantomData};

use hash_hasher::HashedMap;

use crate::{
    internal::{
        active::Insert,
        hash::{self, Hash},
        index,
    },
    Commitment, Position, Proof, Witness,
};

use super::{block, epoch, tree::Tree, InsertError, Tier, TIER_CAPACITY};

/// A builder for an [`Eternity`]: a sequence of epochs, each of which is a sequence of blocks, each
/// of which is a sequence of [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder<Hasher> {
    /// The inner tiers of the builder.
    pub eternity: Tier<Tier<Tier<Commitment, Hasher>, Hasher>, Hasher>,
    /// Hasher to use.
    hasher: PhantomData<Hasher>,
}

impl<Hasher: hash::Hasher> Builder<Hasher> {
    /// Insert a new [`Commitment`] into the [`eternity::Builder`](Builder), returning its [`Position`] if successful.
    ///
    /// See [`crate::Eternity::insert`].
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
            return Err(InsertError::EternityFull);
        }

        // Ensure eternity is not empty
        if self.eternity.is_empty() {
            self.eternity.push_back(Insert::Keep(VecDeque::new()))
        }

        // Calculate the position for the insertion
        let position = self.position();

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

                        // Return the position
                        Ok(position)
                    }
                }
            }
        }
    }

    /// Forget the witness for a given [`Commitment`], returning `true` if it was previously witnessed.
    ///
    /// See [`crate::Eternity::forget`].
    ///
    /// This operation requires a linear scan through the entire builder's contents, and as such
    /// takes time linear in the size of the builder, as opposed to its counterpart,
    ///  [`crate::Eternity::forget`], which is constant time.
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

    /// Insert a block builder's contents as a new block in the current epoch of this [`eternity::Builder`](Builder).
    ///
    /// See [`crate::Eternity::insert_block`].
    pub fn insert_block(&mut self, block: block::Builder<Hasher>) -> Result<(), InsertError> {
        self.insert_block_or_root(Insert::Keep(block))
    }

    /// Insert a block root as a new block root in the current epoch of this [`eternity::Builder`](Builder).
    ///
    /// See [`crate::Eternity::insert_block_root`].
    pub fn insert_block_root(
        &mut self,
        crate::block::Root(block_root): crate::block::Root<Hasher>,
    ) -> Result<(), InsertError> {
        self.insert_block_or_root(Insert::Hash(block_root))
    }

    /// Helper function for inserting a block or block root.
    fn insert_block_or_root(
        &mut self,
        insert: Insert<block::Builder<Hasher>, Hasher>,
    ) -> Result<(), InsertError> {
        // Fail if eternity is full
        if self.eternity.len() >= TIER_CAPACITY {
            return Err(InsertError::EternityFull);
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

                // Insert whatever is to be inserted
                epoch.push_back(insert.map(|block| block.block));
                Ok(())
            }
        }
    }

    /// Calculate the position of the next insertion into this eternity.
    fn position(&self) -> Position {
        let (epoch, block, commitment) = if self.eternity.is_empty() {
            (0.into(), 0.into(), 0.into())
        } else {
            let (block, commitment) = match self.eternity.back().unwrap() {
                Insert::Hash(_) => (index::Block::MAX, index::Commitment::MAX),
                Insert::Keep(epoch) => {
                    if epoch.is_empty() {
                        (0.into(), 0.into())
                    } else {
                        let commitment = match epoch.back().unwrap() {
                            Insert::Hash(_) => index::Commitment::MAX,
                            Insert::Keep(block) => (block.len() as u16).into(),
                        };
                        (((epoch.len() - 1) as u16).into(), commitment)
                    }
                }
            };
            (((self.eternity.len() - 1) as u16).into(), block, commitment)
        };

        Position::from(u64::from(index::within::Eternity {
            epoch,
            block,
            commitment,
        }))
    }

    /// Insert an epoch builder's contents as a new epoch in this [`eternity::Builder`](Builder).
    ///
    /// See [`crate::Eternity::insert_epoch`].
    pub fn insert_epoch(&mut self, epoch: epoch::Builder<Hasher>) -> Result<(), InsertError> {
        if self.eternity.len() < TIER_CAPACITY {
            self.eternity.push_back(Insert::Keep(epoch.epoch));
            Ok(())
        } else {
            Err(InsertError::EternityFull)
        }
    }

    /// Insert an epoch root as a new epoch root in this [`eternity::Builder`](Builder). See
    /// [`crate::Eternity::insert_epoch_root`].
    pub fn insert_epoch_root(
        &mut self,
        crate::epoch::Root(epoch_root): crate::epoch::Root<Hasher>,
    ) -> Result<(), InsertError> {
        if self.eternity.len() < TIER_CAPACITY {
            self.eternity.push_back(Insert::Hash(epoch_root));
            Ok(())
        } else {
            Err(InsertError::EternityFull)
        }
    }

    /// Build an immutable, dense commitment tree, finalizing this builder.
    ///
    /// This is not a mirror of any method on [`crate::Eternity`], because the main crate interface
    /// is incremental, not split into a builder phase and a finalized phase.
    pub fn build(self) -> Eternity<Hasher> {
        let position = self.position();
        let tree = Tree::from_eternity(self.eternity);
        let mut index = HashedMap::default();
        tree.index_with(|commitment, position| {
            index.insert(commitment, position.into());
        });
        Eternity {
            position,
            index,
            tree,
        }
    }
}

/// An immutable, dense, indexed commitment tree.
///
/// This supports all the immutable methods of [`crate::Eternity`].
pub struct Eternity<Hasher> {
    index: HashedMap<Commitment, Position>,
    position: Position,
    tree: Tree<Hasher>,
}

impl<Hasher: hash::Hasher> Eternity<Hasher> {
    /// Get the root hash of this [`Eternity`].
    ///
    /// See [`crate::Eternity::root`].
    pub fn root(&self) -> crate::Root<Hasher> {
        crate::Root(self.tree.root())
    }

    /// Get a [`Proof`] of inclusion for the given [`Commitment`], if it was witnessed.
    ///
    /// See [`crate::Eternity::witness`].
    pub fn witness(&self, commitment: Commitment) -> Option<Proof<Hasher>> {
        let position = *self.index.get(&commitment)?;
        let auth_path = self.tree.witness(position.into());
        Some(Proof::new(commitment, position, auth_path))
    }

    /// Get the block root of the current block of this [`Eternity`], if any.
    ///
    /// See [`crate::Eternity::current_block_root`].
    pub fn current_block_root(&self) -> Option<crate::block::Root<Hasher>> {
        let mut tree = &self.tree;
        for _ in 0..16 {
            if let Tree::Node { children, .. } = tree {
                tree = children.last()?;
            } else {
                return None;
            }
        }
        Some(crate::block::Root(tree.root()))
    }

    /// Get the epoch root of the current epoch of this [`Eternity`], if any.
    ///
    /// See [`crate::Eternity::current_epoch_root`].
    pub fn current_epoch_root(&self) -> Option<crate::epoch::Root<Hasher>> {
        let mut tree = &self.tree;
        for _ in 0..8 {
            if let Tree::Node { children, .. } = tree {
                tree = children.last()?;
            } else {
                return None;
            }
        }
        Some(crate::epoch::Root(tree.root()))
    }

    /// Get the [`Position`] at which the next [`Commitment`] would be inserted.
    ///
    /// See [`crate::Eternity::position`].
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get the number of [`Commitment`]s witnessed in this [`Eternity`].
    ///
    /// See [`crate::Eternity::witnessed_count`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Eternity`] is empty.
    ///
    /// See [`crate::Eternity::is_empty`].
    pub fn is_empty(&self) -> bool {
        if let Tree::Node { ref children, hash } = self.tree {
            hash == Hash::default() && children.is_empty()
        } else {
            false
        }
    }
}
