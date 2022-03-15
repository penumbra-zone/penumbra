use drain_filter_polyfill::VecExt;
use hash_hasher::HashedMap;

use crate::*;

#[path = "block.rs"]
mod block;
pub use block::{Block, BlockMut};

/// A sparse commitment tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536 [`Fq`]s
/// or their [`struct@Hash`]es.
///
/// This is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Epoch {
    pub(super) index: HashedMap<Fq, Vec<index::within::Epoch>>,
    pub(super) inner: Tier<Tier<Item>>,
}

/// A mutable reference to an [`Epoch`] within an [`Eternity`](super::Eternity).
///
/// This supports all the methods of [`Epoch`] that take `&mut self` or `&self`.
#[derive(Debug, PartialEq, Eq)]
pub struct EpochMut<'a> {
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Tier<Item>>,
}

/// A mutable reference to an index from [`Fq`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Eternity`], this index
/// contains all the indices for everything in the tree so far.
#[derive(Debug, PartialEq, Eq)]
pub enum IndexMut<'a> {
    /// An index just for items within an epoch.
    Epoch {
        index: &'a mut HashedMap<Fq, Vec<index::within::Epoch>>,
    },
    /// An index for items within an entire eternity.
    Eternity {
        this_epoch: index::Epoch,
        index: &'a mut HashedMap<Fq, Vec<index::within::Eternity>>,
    },
}

impl Height for Epoch {
    type Height = <Tier<Tier<Item>> as Height>::Height;
}

impl Epoch {
    /// Create a new empty [`Epoch`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an [`EpochMut`] referring to this [`Epoch`].
    pub fn as_mut(&mut self) -> EpochMut {
        EpochMut {
            index: IndexMut::Epoch {
                index: &mut self.index,
            },
            inner: &mut self.inner,
        }
    }

    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Epoch`] if the
    /// [`Epoch`] is full.
    pub fn insert_block(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        self.as_mut().insert_block(block)
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to the most recent [`Block`] of this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted item without adding it to the [`Epoch`] if the
    /// [`Epoch`] is full, or if the most recent [`Block`] is full or was inserted by
    /// [`Insert::Hash`].
    pub fn insert_item(&mut self, block: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.as_mut().insert_item(block)
    }

    /// Forget about the witness for the given [`Fq`].
    ///
    /// Returns `true` if the item was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, item: Fq) -> bool {
        self.as_mut().forget(item)
    }

    /// The total number of [`Fq`]s or [`struct@Hash`]es represented in this [`Epoch`].
    ///
    /// This count includes those which were elided due to a partially filled [`Block`] or summary
    /// root [`struct@Hash`] of a block being inserted.
    ///
    /// In other words, this is `4 ^ 8` times the number of blocks represented in this [`Epoch`],
    /// plus the number of items in the latest block.
    ///
    /// The maximum capacity of an [`Epoch`] is `2 ^ 32`, i.e. `4 ^ 8` blocks of `4 ^ 8` items.
    pub fn len(&self) -> u32 {
        ((self.inner.len() as u32) << 16)
            + match self.inner.focus() {
                None => 0,
                Some(Insert::Hash(_)) => u16::MAX,
                Some(Insert::Keep(block)) => block.len(),
            } as u32
    }

    /// Check whether this [`Epoch`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the root [`struct@Hash`] of this [`Epoch`].
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    pub fn hash(&self) -> Hash {
        self.inner.hash()
    }

    /// Get a [`Proof`] of inclusion for the item at this index in the epoch.
    ///
    /// If the index is not witnessed in this epoch, return `None`.
    pub fn witness(&self, item: Fq) -> Option<Proof<Epoch>> {
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
}

impl EpochMut<'_> {
    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the underlying [`Epoch`]: see
    /// [`Epoch::insert`].
    pub fn insert_block(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        // If we successfully insert this block, here's what its index in the epoch will be:
        let this_block = self.inner.len().into();

        // Decompose the block into its components
        let (block, block_index) = match block {
            Insert::Hash(hash) => (Insert::Hash(hash), Default::default()),
            Insert::Keep(Block { index, inner }) => (Insert::Keep(inner), index),
        };

        // Try to insert the block into the tree, and if successful, track the item and block
        // indices of each item in the inserted block
        if let Err(block) = self.inner.insert(block) {
            Err(block.map(|inner| Block {
                index: block_index,
                inner,
            }))
        } else {
            for (item, indices) in block_index.into_iter() {
                for index::within::Block { item: this_item } in indices {
                    match self.index {
                        IndexMut::Epoch { ref mut index } => {
                            index
                                .entry(item)
                                .or_insert_with(|| Vec::with_capacity(1))
                                .push(index::within::Epoch {
                                    block: this_block,
                                    item: this_item,
                                });
                        }
                        IndexMut::Eternity {
                            this_epoch,
                            ref mut index,
                        } => {
                            index
                                .entry(item)
                                .or_insert_with(|| Vec::with_capacity(1))
                                .push(index::within::Eternity {
                                    epoch: this_epoch,
                                    block: this_block,
                                    item: this_item,
                                });
                        }
                    }
                }
            }

            Ok(())
        }
    }

    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.update(|block| {
            if let Some(block) = block {
                block.insert_item(item)
            } else {
                Err(item)
            }
        })
    }

    /// Forget the witness of the given item, if it was witnessed: see [`Epoch::forget`].
    pub fn forget(&mut self, item: Fq) -> bool {
        let mut forgotten = false;

        match self.index {
            IndexMut::Epoch { ref mut index } => {
                if let Some(within_epoch) = index.get(&item) {
                    // Forget each index for this element in the tree
                    within_epoch.iter().for_each(|&index| {
                        forgotten = true;
                        self.inner.forget(index);
                    });
                    // Remove this entry from the index
                    index.remove(&item);
                }
            }
            IndexMut::Eternity {
                this_epoch,
                ref mut index,
            } => {
                if let Some(within_eternity) = index.get_mut(&item) {
                    // Forget each index within this epoch for this element in the tree
                    VecExt::drain_filter(
                        within_eternity,
                        |index::within::Eternity { epoch, .. }| *epoch == this_epoch,
                    )
                    .for_each(|index| {
                        forgotten = true;
                        self.inner.forget(index);
                    });
                    // Remove this entry from the index if we've removed all its indices
                    if within_eternity.is_empty() {
                        index.remove(&item);
                    }
                }
            }
        }

        forgotten
    }

    /// Update the most recently inserted [`Block`] via methods on [`BlockMut`], and return the
    /// result of the function.
    pub(super) fn update<T>(&mut self, f: impl FnOnce(Option<&mut BlockMut<'_>>) -> T) -> T {
        let this_block = self.inner.len().saturating_sub(1).into();

        let index = match self.index {
            IndexMut::Epoch { ref mut index } => block::IndexMut::Epoch { this_block, index },
            IndexMut::Eternity {
                this_epoch,
                ref mut index,
            } => block::IndexMut::Eternity {
                this_epoch,
                this_block,
                index,
            },
        };

        self.inner.update(|inner| {
            if let Some(inner) = inner {
                if let Insert::Keep(inner) = inner.as_mut() {
                    f(Some(&mut BlockMut { inner, index }))
                } else {
                    f(None)
                }
            } else {
                f(None)
            }
        })
    }
}
