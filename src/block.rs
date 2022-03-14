use std::ops::Deref;

use hash_hasher::HashedMap;

use crate::*;

/// A sparse commitment tree to witness up to 65,536 individual [`Fq`]s or their [`struct@Hash`]es.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub(super) item_index: HashedMap<Fq, index::Item>,
    pub(super) inner: Tier<Item>,
}

/// A mutable reference to a [`Block`] within an [`Epoch`](super::Epoch) or
/// [`Eternity`](super::super::Eternity).
///
/// This supports all the methods of [`Block`] that take `&mut self` or `&self`.
pub struct BlockMut<'a> {
    #[allow(clippy::type_complexity)]
    pub(super) super_index: Option<(
        index::Block,
        &'a mut HashedMap<Fq, index::Block>,
        Option<(index::Epoch, &'a mut HashedMap<Fq, index::Epoch>)>,
    )>,
    block: &'a mut Block,
}

/// [`BlockMut`] implements `Deref<Target = Block>` so it inherits all the *immutable* methods from
/// [`Block`], but crucially it *does not* implemennt `DerefMut`, because the *mutable* methods in
/// `Block` are defined in terms of methods on `BlockMut`.
impl Deref for BlockMut<'_> {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        &*self.block
    }
}

impl Height for Block {
    type Height = <Tier<Item> as Height>::Height;
}

impl Block {
    /// Create a new empty [`Block`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a [`BlockMut`] referring to this [`Block`].
    pub(super) fn as_mut(&mut self) -> BlockMut {
        BlockMut {
            super_index: None,
            block: self,
        }
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to this [`Block`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Block`] if the
    /// block is full.
    pub fn insert(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.as_mut().insert(item)
    }

    /// Forget the witness of the given item, if it was witnessed.
    ///
    /// Returns `true` if the item was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, item: Fq) -> bool {
        self.as_mut().forget(item)
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
    pub fn hash(&self) -> Hash {
        self.inner.hash()
    }

    /// Get a [`Proof`] of inclusion for this item in the block.
    ///
    /// If the index is not witnessed in this block, return `None`.
    pub fn witness(&self, item: Fq) -> Option<Proof<Block>> {
        let index = index::within::Block {
            item: *self.item_index.get(&item)?,
        }
        .into();

        let (auth_path, leaf) = self.inner.witness(index)?;
        debug_assert_eq!(leaf, Hash::of(item));

        Some(Proof {
            index,
            auth_path,
            leaf: item,
        })
    }
}

impl BlockMut<'_> {
    /// Insert into the underlying [`Block`]: see [`Block::insert`].
    pub fn insert(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        // TODO: deal with duplicates

        // If we successfully insert this item, here's what its index in the block will be:
        let this_item = self.len().into();

        // Try to insert the item into the inner tree, and if successful, track the index
        if self.block.inner.insert(item.map(Item::new)).is_err() {
            Err(item)
        } else {
            // Keep track of the item's index in the block, and if applicable, the block's index
            // within its epoch, and if applicable, the epoch's index in the eternity
            if let Some(item) = item.keep() {
                self.block.item_index.insert(item, this_item);
                if let Some((this_block, block_index, epoch_index)) = &mut self.super_index {
                    block_index.insert(item, *this_block);
                    if let Some((this_epoch, epoch_index)) = epoch_index {
                        epoch_index.insert(item, *this_epoch);
                    }
                }
            }
            Ok(())
        }
    }

    /// Forget the witness of the given item, if it was witnessed: see [`Block::forget`].
    pub fn forget(&mut self, item: Fq) -> bool {
        if let Some(this_item) = self.item_index.get(&item) {
            // If this `EpochMut` refers to a containing `Eternity`, it could be that the item
            // doesn't belong to this epoch, but rather another one: check this before proceeding
            if let Some((this_block, block_index, epoch_index)) = &self.super_index {
                let correct_block = this_block
                    == block_index
                        .get(&item)
                        .expect("if block index contains item, then epoch index must contain item");
                if !correct_block {
                    return false;
                }
                if let Some((this_epoch, epoch_index)) = epoch_index {
                    let correct_epoch = this_epoch
                        == epoch_index.get(&item).expect(
                            "if block index contains item, then epoch index must contain item",
                        );
                    if !correct_epoch {
                        return false;
                    }
                }
            }

            // Calculate the index for the item
            let index = index::within::Block { item: *this_item };

            // Forget the item from the inner tree
            let forgotten = self.block.inner.forget(index);

            // The index should never contain things that aren't witnessed
            debug_assert!(forgotten, "indexed item must be witnessed in tree");

            // Remove the item from all indices
            self.block.item_index.remove(&item);
            if let Some((_, block_index, epoch_index)) = &mut self.super_index {
                block_index.remove(&item);
                if let Some((_, epoch_index)) = epoch_index {
                    epoch_index.remove(&item);
                }
            }

            // The item was indeed previously present, now forgotten
            true
        } else {
            false
        }
    }
}
