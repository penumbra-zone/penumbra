use hash_hasher::HashedMap;

use crate::*;

#[path = "epoch.rs"]
mod epoch;
pub use epoch::{Block, BlockMut, Epoch, EpochMut};

/// A sparse commitment tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Fq`]s or their [`struct@Hash`]es.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Eternity {
    index: HashedMap<Fq, Vec<index::within::Eternity>>,
    inner: Tier<Tier<Tier<Item>>>,
}

impl Height for Eternity {
    type Height = <Tier<Tier<Tier<Item>>> as Height>::Height;
}

impl Eternity {
    /// Create a new empty [`Eternity`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Epoch`] (or its root hash) all at once to this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(epoch)` without adding it to the [`Eternity`] if the [`Eternity`] is full.
    pub fn insert_epoch(&mut self, epoch: Insert<Epoch>) -> Result<(), Insert<Epoch>> {
        // If we successfully insert this epoch, here's what its index in the epoch will be:
        let this_epoch = self.inner.len().into();

        // Decompose the block into its components
        let (epoch, epoch_index) = match epoch {
            Insert::Hash(hash) => (Insert::Hash(hash), Default::default()),
            Insert::Keep(Epoch { index, inner }) => (Insert::Keep(inner), index),
        };

        // Try to insert the block into the tree, and if successful, track the item, block, and
        // epoch indices of each inserted item
        if let Err(epoch) = self.inner.insert(epoch) {
            Err(epoch.map(|inner| Epoch {
                index: epoch_index,
                inner,
            }))
        } else {
            for (item, indices) in epoch_index.into_iter() {
                for index::within::Epoch {
                    item: this_item,
                    block: this_block,
                } in indices
                {
                    self.index
                        .entry(item)
                        .or_insert_with(|| Vec::with_capacity(1))
                        .push(index::within::Eternity {
                            epoch: this_epoch,
                            block: this_block,
                            item: this_item,
                        });
                }
            }

            Ok(())
        }
    }

    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the most recently inserted
    /// [`Epoch`] of this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Eternity`] if
    /// the [`Eternity`] is full, or the most recently inserted [`Epoch`] is full or was inserted by
    /// [`Insert::Hash`].
    pub fn insert_block(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        // If the eternity is empty, we need to create a new epoch to insert the block into
        if self.inner.is_empty() && self.insert_epoch(Insert::Keep(Epoch::new())).is_err() {
            return Err(block);
        }

        self.update(|epoch| {
            if let Some(epoch) = epoch {
                epoch.insert_block(block)
            } else {
                Err(block)
            }
        })
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to the most recent [`Block`] of the most recent
    /// [`Epoch`] of this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Eternity`] if
    /// the [`Eternity`] is full, or the most recently inserted [`Epoch`] is full or was inserted by
    /// [`Insert::Hash`], or the most recently inserted [`Block`] is full or was inserted by
    /// [`Insert::Hash`].
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        // If the eternity is empty, we need to create a new epoch to insert the item into
        if self.inner.is_empty() && self.insert_epoch(Insert::Keep(Epoch::new())).is_err() {
            return Err(item);
        }

        self.update(|epoch| {
            if let Some(epoch) = epoch {
                epoch.insert_item(item)
            } else {
                Err(item)
            }
        })
    }

    /// The total number of [`Fq`]s or [`struct@Hash`]es represented in this [`Epoch`].
    ///
    /// This count includes those items which are elided due to a partially filled [`Block`] or
    /// [`Epoch`], or summary root [`struct@Hash`] of a block or epoch being inserted.
    ///
    /// The maximum capacity of an [`Eternity`] is 281,474,976,710,656 = 65,536 [`Epoch`]s of 65,536
    /// [`Block`]s of 65,536 [`Fq`]s.
    pub fn len(&self) -> u64 {
        ((self.inner.len() as u64) << 32)
            + (match self.inner.focus() {
                None => 0,
                Some(Insert::Hash(_)) => u32::MAX,
                Some(Insert::Keep(epoch)) => {
                    (match epoch.focus() {
                        None => 0,
                        Some(Insert::Hash(_)) => u16::MAX,
                        Some(Insert::Keep(block)) => block.len(),
                    }) as u32
                }
            } << 16) as u64
    }

    /// Check whether this [`Eternity`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the root [`struct@Hash`] of this [`Eternity`].
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    pub fn root(&self) -> Hash {
        self.inner.hash()
    }

    /// Get a [`Proof`] of inclusion for the item at this index in the eternity.
    ///
    /// If the index is not witnessed in this eternity, return `None`.
    pub fn witness(&self, item: Fq) -> Option<Proof<Eternity>> {
        let index = *self
            .index
            .get(&item)?
            .last()
            .expect("vector of indices is non-empty");

        let (auth_path, leaf) = self.inner.witness(index)?;
        debug_assert_eq!(leaf, Hash::of(item));

        Some(Proof {
            index: index.into(),
            auth_path,
            leaf: item,
        })
    }

    /// Forget about the witness for the given [`Fq`].
    ///
    /// Returns `true` if the item was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, item: Fq) -> bool {
        let mut forgotten = false;

        if let Some(within_epoch) = self.index.get(&item) {
            // Forget each index for this element in the tree
            within_epoch.iter().for_each(|&index| {
                forgotten = true;
                self.inner.forget(index);
            });
            // Remove this entry from the index
            self.index.remove(&item);

            // The item was indeed previously present, now forgotten
            true
        } else {
            false
        }
    }

    /// Update the most recently inserted [`Epoch`] via methods on [`EpochMut`], and return the
    /// result of the function.
    fn update<T>(&mut self, f: impl FnOnce(Option<&mut EpochMut<'_>>) -> T) -> T {
        let this_epoch = self.inner.len().saturating_sub(1).into();

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
