use hash_hasher::HashedMap;
use serde::{Deserialize, Serialize};

use crate::internal::{active::Forget as _, path::Witness as _};
use crate::*;

#[path = "epoch.rs"]
pub(crate) mod epoch;
use epoch::{block, block::Block, Epoch, EpochMut};

mod proof;
pub use proof::{Proof, VerifyError};

pub mod error;
pub use error::{
    InsertBlockError, InsertBlockRootError, InsertEpochError, InsertEpochRootError, InsertError,
};

/// A sparse merkle tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Eternity {
    index: HashedMap<Commitment, index::within::Eternity>,
    inner: Tier<Tier<Tier<Item>>>,
}

/// The root hash of an [`Eternity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Root(Hash);

impl From<Root> for Hash {
    fn from(root: Root) -> Self {
        root.0
    }
}

impl Height for Eternity {
    type Height = <Tier<Tier<Tier<Item>>> as Height>::Height;
}

impl Eternity {
    /// Create a new empty [`Eternity`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the root hash of this [`Eternity`].
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    pub fn root(&self) -> Root {
        Root(self.inner.hash())
    }

    /// Add a new [`Commitment`] to the most recent [`Block`] of the most recent [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if the [`Eternity`] is full, or the most recently inserted [`Epoch`]
    /// is full or was inserted by [`insert_epoch_root`](Eternity::insert_epoch_root), or the most
    /// recently inserted [`Block`] is full or was inserted by
    /// [`insert_block_root`](Eternity::insert_block_root).
    pub fn insert(&mut self, witness: Witness, commitment: Commitment) -> Result<(), InsertError> {
        self.insert_commitment_or_root(match witness {
            Keep => Insert::Keep(commitment),
            Forget => Insert::Hash(Hash::of(commitment)),
        })
    }

    /// Get a [`Proof`] of inclusion for the commitment at this index in the eternity.
    ///
    /// If the index is not witnessed in this eternity, return `None`.
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let index = *self.index.get(&commitment)?;

        let (auth_path, leaf) = self.inner.witness(index)?;
        debug_assert_eq!(leaf, Hash::of(commitment));

        Some(Proof(crate::proof::Proof {
            index: index.into(),
            auth_path,
            leaf: commitment,
        }))
    }

    /// Forget about the witness for the given [`Commitment`].
    ///
    /// Returns `true` if the commitment was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;

        if let Some(&within_epoch) = self.index.get(&commitment) {
            // We forgot something
            forgotten = true;
            // Forget the index for this element in the tree
            let forgotten = self.inner.forget(within_epoch);
            debug_assert!(forgotten);
            // Remove this entry from the index
            self.index.remove(&commitment);
        }

        forgotten
    }

    /// Insert an commitment or its root (helper function for [`insert`].
    fn insert_commitment_or_root(
        &mut self,
        commitment: Insert<Commitment>,
    ) -> Result<(), InsertError> {
        // If the eternity is empty, we need to create a new epoch to insert the commitment into
        if self.inner.is_empty() && self.insert_epoch(Epoch::new()).is_err() {
            return Err(InsertError::Full);
        }

        match self.update(|epoch| {
            if let Some(epoch) = epoch {
                epoch.insert(commitment).map_err(|err| match err {
                    epoch::InsertError::Full => InsertError::EpochFull,
                    epoch::InsertError::BlockFull => InsertError::BlockFull,
                    epoch::InsertError::BlockForgotten => InsertError::BlockForgotten,
                })
            } else {
                Err(InsertError::EpochForgotten)
            }
        }) {
            Err(err) => Err(err),
            Ok(None) => Ok(()),
            Ok(Some(replaced)) => {
                // If inserting this commitment replaced some other commitment, forget the replaced index
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
                Ok(())
            }
        }
    }

    /// Add a new [`Block`] all at once to the most recently inserted [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockError`] containing the inserted block without adding it to the
    /// [`Eternity`] if the [`Eternity`] is full, or the most recently inserted [`Epoch`] is full or
    /// was inserted by [`Insert::Hash`].
    pub fn insert_block(&mut self, block: Block) -> Result<(), InsertBlockError> {
        // If the eternity is empty, we need to create a new epoch to insert the block into
        if self.inner.is_empty() && self.insert_epoch(Epoch::new()).is_err() {
            return Err(InsertBlockError::Full(block));
        }

        match self.update(|epoch| {
            if let Some(epoch) = epoch {
                epoch
                    .insert_block_or_root(Insert::Keep(block))
                    .map_err(|insert| {
                        if let Insert::Keep(block) = insert {
                            InsertBlockError::EpochFull(block)
                        } else {
                            unreachable!(
                                "failing to insert a block always returns the original block"
                            )
                        }
                    })
            } else {
                Err(InsertBlockError::EpochForgotten(block))
            }
        }) {
            Err(err) => Err(err),
            Ok(replaced) => {
                // When inserting the block, some indices in the block may overwrite existing
                // indices; we now can forget those indices because they're inaccessible
                for replaced in replaced {
                    let forgotten = self.inner.forget(replaced);
                    debug_assert!(forgotten);
                }
                Ok(())
            }
        }
    }

    /// Add the root hash of an [`Block`] to this [`Eternity`], without inserting any of the
    /// witnessed commitments in that [`Block`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockRootError`] if the [`Eternity`] is full, or the most recently inserted
    /// [`Epoch`] is full or was inserted by [`insert_epoch_root`](Eternity::insert_epoch_root).
    pub fn insert_block_root(
        &mut self,
        block_root: block::Root,
    ) -> Result<(), InsertBlockRootError> {
        // If the eternity is empty, we need to create a new epoch to insert the block into
        if self.inner.is_empty() && self.insert_epoch(Epoch::new()).is_err() {
            return Err(InsertBlockRootError::Full);
        }

        match self.update(|epoch| {
            if let Some(epoch) = epoch {
                epoch
                    .insert_block_or_root(Insert::Hash(block_root.0))
                    .map_err(|_| InsertBlockRootError::EpochFull)
            } else {
                Err(InsertBlockRootError::EpochForgotten)
            }
        }) {
            Err(err) => Err(err),
            Ok(replaced) => {
                // When inserting the block, some indices in the block may overwrite existing
                // indices; we now can forget those indices because they're inaccessible
                for replaced in replaced {
                    let forgotten = self.inner.forget(replaced);
                    debug_assert!(forgotten);
                }
                Ok(())
            }
        }
    }

    /// Get the root hash of the most recent [`Block`] in the most recent [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// If the [`Eternity`] is empty or the most recent [`Epoch`] was inserted with
    /// [`Eternity::insert_epoch_root`], returns `None`.
    pub fn current_block_root(&self) -> Option<block::Root> {
        self.inner.focus().and_then(|epoch| {
            epoch
                .as_ref()
                .keep()?
                .focus()
                .map(|block| block::Root(block.hash()))
        })
    }

    /// Add a new [`Epoch`] all at once to this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertEpochError`] containing the epoch without adding it to the [`Eternity`] if
    /// the [`Eternity`] is full.
    pub fn insert_epoch(&mut self, epoch: Epoch) -> Result<(), InsertEpochError> {
        self.insert_epoch_or_root(Insert::Keep(epoch))
            .map_err(|insert| {
                if let Insert::Keep(epoch) = insert {
                    InsertEpochError(epoch)
                } else {
                    unreachable!("failing to insert an epoch always returns the original epoch")
                }
            })
    }

    /// Add the root hash of an [`Epoch`] to this [`Eternity`], without inserting any of the
    /// witnessed commitments in that [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertEpochRootError`] if the [`Eternity`] is full.
    pub fn insert_epoch_root(
        &mut self,
        epoch_root: epoch::Root,
    ) -> Result<(), InsertEpochRootError> {
        self.insert_epoch_or_root(Insert::Hash(epoch_root.0))
            .map_err(|insert| {
                if let Insert::Hash(_) = insert {
                    InsertEpochRootError
                } else {
                    unreachable!("failing to insert an epoch root always returns the original root")
                }
            })
    }

    /// Insert an epoch or its root (helper function for [`insert_epoch`] and [`insert_epoch_root`]).
    fn insert_epoch_or_root(&mut self, epoch: Insert<Epoch>) -> Result<(), Insert<Epoch>> {
        // If we successfully insert this epoch, here's what its index in the epoch will be:
        let this_epoch = self.inner.position().into();

        // Decompose the block into its components
        let (epoch, epoch_index) = match epoch {
            Insert::Hash(hash) => (Insert::Hash(hash), Default::default()),
            Insert::Keep(Epoch { index, inner }) => (Insert::Keep(inner), index),
        };

        // Try to insert the block into the tree, and if successful, track the commitment, block, and
        // epoch indices of each inserted commitment
        if let Err(epoch) = self.inner.insert(epoch) {
            Err(epoch.map(|inner| Epoch {
                index: epoch_index,
                inner,
            }))
        } else {
            for (
                commitment,
                index::within::Epoch {
                    block: this_block,
                    commitment: this_commitment,
                },
            ) in epoch_index.into_iter()
            {
                if let Some(replaced) = self.index.insert(
                    commitment,
                    index::within::Eternity {
                        epoch: this_epoch,
                        block: this_block,
                        commitment: this_commitment,
                    },
                ) {
                    // Forget the previous index of this inserted epoch, if there was one
                    self.inner.forget(replaced);
                }
            }

            Ok(())
        }
    }

    /// Get the root hash of the most recent [`Epoch`] in this [`Eternity`].
    ///
    /// If the [`Eternity`] is empty, returns `None`.
    pub fn current_epoch_root(&self) -> Option<epoch::Root> {
        self.inner.focus().map(|epoch| epoch::Root(epoch.hash()))
    }

    /// The position in this [`Eternity`] at which the next [`Commitment`] would be inserted.
    ///
    /// The maximum capacity of an [`Eternity`] is 281,474,976,710,656 = 65,536 [`Epoch`]s of 65,536
    /// [`Block`]s of 65,536 [`Commitment`]s.
    ///
    /// Note that [`forget`](Eternity::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Eternity::witnessed_count).
    pub fn position(&self) -> u64 {
        ((self.inner.position() as u64) << 32)
            + (match self.inner.focus() {
                None => 0,
                Some(Insert::Hash(_)) => u32::MAX,
                Some(Insert::Keep(epoch)) => {
                    (match epoch.focus() {
                        None => 0,
                        Some(Insert::Hash(_)) => u16::MAX,
                        Some(Insert::Keep(block)) => block.position(),
                    }) as u32
                }
            } << 16) as u64
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Eternity`].
    ///
    /// Note that [`forget`](Eternity::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Eternity::position) of the next inserted [`Commitment`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Eternity`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Update the most recently inserted [`Epoch`] via methods on [`EpochMut`], and return the
    /// result of the function.
    fn update<T>(&mut self, f: impl FnOnce(Option<&mut EpochMut<'_>>) -> T) -> T {
        let this_epoch = self.inner.position().saturating_sub(1).into();

        let index = epoch::IndexMut::Eternity {
            this_epoch,
            index: &mut self.index,
        };

        self.inner.update(|inner| {
            if let Some(inner) = inner {
                if let Insert::Keep(inner) = inner.as_mut() {
                    f(Some(&mut EpochMut { inner, index }))
                } else {
                    f(None)
                }
            } else {
                f(None)
            }
        })
    }
}
