//! The tiered commitment tree for Penumbra.

#![warn(missing_docs)]

#[macro_use]
extern crate derivative;

use std::fmt::Debug;

pub mod internal;
mod item;

#[doc(inline)]
pub use item::Item;

#[doc(inline)]
pub use internal::{active::Insert, hash::Hash};

use internal::{
    active::{Active, Focus, Full, Tier},
    complete::Complete,
    hash::GetHash,
    height::Height,
};

/// A sparse commitment tree to store up to 65,536 [`Epoch`]s, each containing up to 65,536
/// [`Block`]s, each containing up to 65,536 `Item`s or their [`struct@Hash`]es.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derivative(Debug(bound = "Item: Debug, Item::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, Item::Complete: Clone"))]
#[derivative(PartialEq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
pub struct Eternity<Item: Focus> {
    epochs_witnessed: u16,
    blocks_witnessed: u16,
    items_witnessed: u16,
    len: u64,
    inner: Tier<Tier<Tier<Item>>>,
}

impl<Item: Focus> Eternity<Item> {
    /// Create a new empty [`Eternity`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Epoch`] (or its root hash) all at once to this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(epoch)` without modifying the [`Eternity`] if the [`Eternity`] is full.
    pub fn insert_epoch(&mut self, epoch: Insert<Epoch<Item>>) -> Result<(), Insert<Epoch<Item>>> {
        let (blocks_witnessed, items_witnessed, len) = match epoch {
            Insert::Keep(ref epoch) => (
                epoch.blocks_witnessed(),
                epoch.items_witnessed(),
                epoch.len(),
            ),
            Insert::Hash(_) => (0, 0, 0),
        };

        let result = self
            .inner
            .insert(epoch.map(|epoch| epoch.inner))
            .map_err(|inner| {
                inner.map(|inner| Epoch {
                    blocks_witnessed,
                    items_witnessed,
                    len,
                    inner,
                })
            });

        if result.is_ok() {
            // The start index of the current epoch (mask off the last 32 bits)
            let epoch_start = self.len & ((1u16 as u64) << 32);
            // The size of each block (2^32)
            let epoch_size = 1 << 32;
            // The new length is the start index of the *next* block plus the size of the one being added
            self.len = epoch_start + epoch_size + (len as u64);

            self.epochs_witnessed += 1;
            self.blocks_witnessed += blocks_witnessed;
            self.items_witnessed += items_witnessed;
        }

        result
    }

    /// The total number of [`Epoch`]s witnessed in this [`Eternity`].
    pub fn epochs_witnessed(&self) -> u16 {
        self.epochs_witnessed
    }

    /// Add a new [`Block`] (or its root hash) all at once to the current [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Eternity`] if:
    ///
    /// 1. we have reached the end of time (i.e. the [`Eternity`] is full),
    /// 2. the current [`Epoch`] is full, or
    /// 3. the current [`Epoch`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_block(&mut self, block: Insert<Block<Item>>) -> Result<(), Insert<Block<Item>>> {
        let (items_witnessed, len) = match block {
            Insert::Keep(ref block) => (block.items_witnessed(), block.len()),
            Insert::Hash(_) => (0, 0),
        };

        // Mutable container for the thing to be inserted: we will take it out of here if the
        // closure is run, but if it isn't, we need to recover it
        let mut block = Some(block);

        let result = self
            .inner
            .update(|focus| {
                // The closure is being run, so we take the item to insert
                let block = block.take().unwrap();

                if let Insert::Keep(focus) = focus {
                    focus
                        .insert(block.map(|block| block.inner))
                        .map_err(|result| result.map(|inner| Block { inner }))
                } else {
                    Err(block)
                }
            })
            // In this case, the closure was never invoked, so we can take the item back here
            .unwrap_or_else(|| Err(block.take().unwrap()));

        if result.is_ok() {
            // The start index of the current block (mask off the last 16 bits)
            let block_start = self.len & ((1u16 as u64) << 16);
            // The size of each block (2^16)
            let block_size = 1 << 16;
            // The new length is the start index of the *next* block plus the size of the one being added
            self.len = block_start + block_size + (len as u64);

            self.blocks_witnessed += 1;
            self.items_witnessed += items_witnessed;
        }

        result
    }

    /// The total number of [`Block`]s witnessed in every [`Epoch`] in this [`Eternity`].
    pub fn blocks_witnessed(&self) -> u16 {
        self.blocks_witnessed
    }

    /// Add a new `Item` (or its hash) to the current [`Block`] of the current [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Eternity`] if:
    ///
    /// 1. we have reached the end of time (i.e. the [`Eternity`] is full),
    /// 2. the current [`Epoch`] is full,
    /// 3. the current [`Epoch`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion,
    /// 4. the current [`Block`] is full, or
    /// 5. the current [`Block`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_item(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        // Mutable container for the thing to be inserted: we will take it out of here if the
        // closure is run, but if it isn't, we need to recover it
        let mut item = Some(item);

        let result = self
            .inner
            .update(|focus| {
                if let Insert::Keep(focus) = focus {
                    focus
                        .update(|focus| {
                            // The closure is being run, so we take the item to insert
                            let item = item.take().unwrap();

                            if let Insert::Keep(focus) = focus {
                                focus.insert(item)
                            } else {
                                Err(item)
                            }
                        })
                        // In this case, the closure was never invoked, so we can take the item back here
                        .unwrap_or_else(|| Err(item.take().unwrap()))
                } else {
                    Err(item.take().unwrap())
                }
            })
            // In this case, the closure was never invoked, so we can take the item back here
            .unwrap_or_else(|| Err(item.take().unwrap()));

        if result.is_ok() {
            self.len += 1;
            self.items_witnessed += 1;
        }

        result
    }

    /// The total number of [`Item`]s witnessed in every [`Block`] in every [`Epoch`] in this [`Eternity`].
    pub fn items_witnessed(&self) -> u16 {
        self.items_witnessed
    }

    /// The total number of [`Item`]s or [`struct@Hash`]es represented in this [`Epoch`].
    ///
    /// This count includes those which were elided due to a partially filled [`Block`] or
    /// [`Epoch`], or summary root [`struct@Hash`] of a block or epoch being inserted. In other words, this
    /// is `2 ^ 32` times the number of epochs represented in this [`Eternity`], plus `2 ^ 16` times
    /// the number of blocks represented in this [`Eternity`], plus the number of items in the
    /// latest block.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Check whether this [`Eternity`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// A sparse commitment tree to store up to 65,536 [`Block`]s, each containing up to 65,536 `Item`s
/// or their [`struct@Hash`]es.
///
/// This is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derivative(Debug(bound = "Item: Debug, Item::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, Item::Complete: Clone"))]
#[derivative(PartialEq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
pub struct Epoch<Item: Focus> {
    blocks_witnessed: u16,
    items_witnessed: u16,
    len: u32,
    inner: Tier<Tier<Item>>,
}

impl<Item: Focus> Epoch<Item> {
    /// Create a new empty [`Epoch`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Block`] (or its root hash) all at once to the current [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Epoch`] if the
    /// [`Epoch`] is full.
    pub fn insert_block(&mut self, block: Insert<Block<Item>>) -> Result<(), Insert<Block<Item>>> {
        let (items_witnessed, len) = match block {
            Insert::Keep(ref block) => (block.items_witnessed(), block.len()),
            Insert::Hash(_) => (0, 0),
        };

        let result = self
            .inner
            .insert(block.map(|block| block.inner))
            .map_err(|inner| inner.map(|inner| Block { inner }));

        if result.is_ok() {
            // The start index of the current block (mask off the last 16 bits)
            let block_start = self.len & ((1u16 as u32) << 16);
            // The size of each block (2^16)
            let block_size = 1 << 16;
            // The new length is the start index of the *next* block plus the size of the one being added
            self.len = block_start + block_size + (len as u32);

            self.blocks_witnessed += 1;
            self.items_witnessed += items_witnessed;
        }

        result
    }

    /// The number of [`Block`]s witnessed in this [`Epoch`].
    pub fn blocks_witnessed(&self) -> u16 {
        self.blocks_witnessed
    }

    /// Add a new `Item` (or its hash) to the current [`Block`] of this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Epoch`] if:
    ///
    /// 1. the [`Epoch`] is full,
    /// 2. the current [`Block`] is full, or
    /// 3. the current [`Block`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_item(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        // Mutable container for the thing to be inserted: we will take it out of here if the
        // closure is run, but if it isn't, we need to recover it
        let mut item = Some(item);

        let result = self
            .inner
            .update(|focus| {
                // The closure is being run, so we take the item to insert
                let item = item.take().unwrap();

                if let Insert::Keep(focus) = focus {
                    focus.insert(item)
                } else {
                    Err(item)
                }
            })
            // In this case, the closure was never invoked, so we can take the item back here
            .unwrap_or_else(|| Err(item.take().unwrap()));

        if result.is_ok() {
            self.items_witnessed += 1;
            self.len += 1;
        }

        result
    }

    /// The total number of [`Item`]s witnessed in every [`Block`] in this [`Epoch`].
    pub fn items_witnessed(&self) -> u16 {
        self.items_witnessed
    }

    /// The total number of [`Item`]s or [`struct@Hash`]es represented in this [`Epoch`].
    ///
    /// This count includes those which were elided due to a partially filled [`Block`] or summary
    /// root [`struct@Hash`] of a block being inserted. In other words, this is `2 ^ 16` times the number
    /// of blocks represented in this [`Epoch`], plus the number of items in the latest block.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Check whether this [`Epoch`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// A sparse commitment tree to store up to 65,536 individual `Item`s or their [`struct@Hash`]es.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derivative(Debug(bound = "Item: Debug, Item::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, Item::Complete: Clone"))]
#[derivative(PartialEq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
#[derivative(Eq(bound = "Item: Eq + PartialEq<Item::Complete>, Item::Complete: Eq"))]
pub struct Block<Item: Focus> {
    inner: Tier<Item>,
}

impl<Item: Focus> Block<Item> {
    /// Create a new empty [`Block`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new item or its hash to this [`Block`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Block`] if the
    /// block is full.
    pub fn insert_item(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        self.inner.insert(item)
    }

    /// The number of items witnessed in this [`Block`].
    pub fn items_witnessed(&self) -> u16 {
        self.inner.size()
    }

    /// The total number of [`Item`]s or [`struct@Hash`]es represented in this [`Block`].
    pub fn len(&self) -> u16 {
        self.inner.len()
    }

    /// Check whether this [`Block`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
