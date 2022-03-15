use hash_hasher::HashedMap;

use crate::*;

/// A sparse commitment tree to witness up to 65,536 individual [`Fq`]s or their [`struct@Hash`]es.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub(super) index: HashedMap<Fq, index::within::Block>,
    pub(super) inner: Tier<Item>,
}

/// A mutable reference to a [`Block`].
#[derive(Debug, PartialEq, Eq)]
pub struct BlockMut<'a> {
    pub(super) index: IndexMut<'a>,
    pub(super) inner: &'a mut Tier<Item>,
}

/// A mutable reference to an index from [`Fq`] to indices into a tree.
///
/// When a [`BlockMut`] is derived from some containing [`Epoch`] or [`Eternity`], this index
/// contains all the indices for everything in the tree so far.
#[derive(Debug, PartialEq, Eq)]
pub enum IndexMut<'a> {
    /// An index just for items within a block.
    Block {
        index: &'a mut HashedMap<Fq, index::within::Block>,
    },
    /// An index just for items within an epoch.
    Epoch {
        this_block: index::Block,
        index: &'a mut HashedMap<Fq, index::within::Epoch>,
    },
    /// An index for items within an entire eternity.
    Eternity {
        this_epoch: index::Epoch,
        this_block: index::Block,
        index: &'a mut HashedMap<Fq, index::within::Eternity>,
    },
}

/// An overwritten index which should be forgotten.
#[derive(Debug, PartialEq, Eq)]
pub enum ReplacedIndex {
    /// An index from within an epoch.
    Epoch(index::within::Epoch),
    /// An index from within an entire eternity.
    Eternity(index::within::Eternity),
}

impl Height for Block {
    type Height = <Tier<Item> as Height>::Height;
}

impl Block {
    /// Create a new empty [`Block`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a [`BlockMut`] from this [`Block`].
    pub(super) fn as_mut(&mut self) -> BlockMut {
        BlockMut {
            index: IndexMut::Block {
                index: &mut self.index,
            },
            inner: &mut self.inner,
        }
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to this [`Block`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Block`] if the
    /// block is full.
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.as_mut().insert_item(item).map(|option|
                // We shouldn't ever be handing back a replaced index here, because the index should
                // be forgotten internally to the method when the block is not owned by a larger structure
                debug_assert!(option.is_none()))
    }

    /// The total number of [`Fq`]s or [`struct@Hash`]es represented in the underlying [`Block`].
    pub fn len(&self) -> u16 {
        self.inner.len()
    }

    /// Check whether the underlying [`Block`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the root [`struct@Hash`] of this [`Block`].
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

    /// Get a [`Proof`] of inclusion for this item in the block.
    ///
    /// If the index is not witnessed in this block, return `None`.
    pub fn witness(&self, item: Fq) -> Option<Proof<Block>> {
        let index = *self.index.get(&item)?;

        let (auth_path, leaf) = self.inner.witness(index)?;
        debug_assert_eq!(leaf, Hash::of(item));

        Some(Proof {
            index: index.into(),
            auth_path,
            leaf: item,
        })
    }

    /// Forget the witness of the given item, if it was witnessed.
    ///
    /// Returns `true` if the item was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, item: Fq) -> bool {
        self.as_mut().forget(item)
    }
}

impl BlockMut<'_> {
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<Option<ReplacedIndex>, Insert<Fq>> {
        // If we successfully insert this item, here's what its index in the block will be:
        let this_item: index::Item = self.inner.len().into();

        // Try to insert the item into the inner tree, and if successful, track the index
        if self.inner.insert(item.map(Item::new)).is_err() {
            Err(item)
        } else {
            // Keep track of the item's index in the block, and if applicable, the block's index
            // within its epoch, and if applicable, the epoch's index in the eternity
            if let Insert::Keep(item) = item {
                match self.index {
                    IndexMut::Block { ref mut index } => {
                        if let Some(replaced) =
                            index.insert(item, index::within::Block { item: this_item })
                        {
                            self.inner.forget(replaced);
                        }
                        Ok(None)
                    }
                    IndexMut::Epoch {
                        this_block,
                        ref mut index,
                    } => Ok(index
                        .insert(
                            item,
                            index::within::Epoch {
                                block: this_block,
                                item: this_item,
                            },
                        )
                        .map(ReplacedIndex::Epoch)),
                    IndexMut::Eternity {
                        this_epoch,
                        this_block,
                        ref mut index,
                    } => Ok(index
                        .insert(
                            item,
                            index::within::Eternity {
                                epoch: this_epoch,
                                block: this_block,
                                item: this_item,
                            },
                        )
                        .map(ReplacedIndex::Eternity)),
                }
            } else {
                Ok(None)
            }
        }
    }

    pub fn forget(&mut self, item: Fq) -> bool {
        let mut forgotten = false;

        match self.index {
            IndexMut::Block { ref mut index } => {
                if let Some(&within_block) = index.get(&item) {
                    // We forgot something
                    forgotten = true;
                    // Forget the index for this element in the tree
                    let forgotten = self.inner.forget(within_block);
                    debug_assert!(forgotten);
                    // Remove this entry from the index
                    index.remove(&item);
                }
            }
            IndexMut::Epoch {
                this_block,
                ref mut index,
            } => {
                if let Some(&within_epoch) = index.get(&item) {
                    // Only forget this index if it belongs to the current block
                    if within_epoch.block == this_block {
                        // We forgot something
                        forgotten = true;
                        // Forget the index for this element in the tree
                        let forgotten = self.inner.forget(within_epoch);
                        debug_assert!(forgotten);
                        // Remove this entry from the index
                        index.remove(&item);
                    }
                }
            }
            IndexMut::Eternity {
                this_epoch,
                this_block,
                ref mut index,
            } => {
                if let Some(&within_eternity) = index.get(&item) {
                    // Only forget this index if it belongs to the current block and that block
                    // belongs to the current epoch
                    if within_eternity.block == this_block && within_eternity.epoch == this_epoch {
                        // We forgot something
                        forgotten = true;
                        // Forget the index for this element in the tree
                        let forgotten = self.inner.forget(within_eternity);
                        debug_assert!(forgotten);
                        // Remove this entry from the index
                        index.remove(&item);
                    }
                }
            }
        }

        forgotten
    }
}
