use crate::*;

/// A sparse commitment tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Fq`]s or their [`struct@Hash`]es.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Eternity {
    epochs_witnessed: u16,
    blocks_witnessed: u16,
    items_witnessed: u16,
    len: u64,
    inner: Tier<Tier<Tier<Hash>>>,
}

impl Height for Eternity {
    type Height = <Tier<Tier<Tier<Hash>>> as Height>::Height;
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
            let epoch_start = self.len & (!(u32::MAX as u64));
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

    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the current [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Eternity`] if:
    ///
    /// 1. the [`Eternity`] is full,
    /// 2. the current [`Epoch`] is full, or
    /// 3. the current [`Epoch`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_block(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        // If the eternity is empty, we need to insert a new `Epoch` before we can insert a `Block`
        // into it
        let initialized = if self.inner.is_empty() {
            if self.inner.insert(Insert::Keep(Tier::default())).is_err() {
                return Err(block);
            } else {
                true
            }
        } else {
            false
        };

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
            if initialized {
                self.epochs_witnessed += 1;
            }

            // The start index of the current block (mask off the last 16 bits)
            let block_start = self.len & (!(u16::MAX as u64));
            // The size of each block (2^16)
            let block_size = 1 << 16;
            // The start of the next block
            let next_block_start = if !initialized {
                // If we just initialized this, we shouldn't increment the length to the next block
                0
            } else {
                // Otherwise, we need to move forward to the start of the next block
                block_start + block_size
            };

            // The new length is the start index of the *next* block plus the size of the one being added
            self.len = next_block_start + (len as u64);

            self.blocks_witnessed += 1;
            self.items_witnessed += items_witnessed;
        }

        result
    }

    /// The total number of [`Block`]s witnessed in every [`Epoch`] in this [`Eternity`].
    pub fn blocks_witnessed(&self) -> u16 {
        self.blocks_witnessed
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to the current [`Block`] of the current [`Epoch`] of
    /// this [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Eternity`] if:
    ///
    /// 1. the [`Eternity`] is full,
    /// 2. the current [`Epoch`] is full,
    /// 3. the current [`Epoch`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion,
    /// 4. the current [`Block`] is full, or
    /// 5. the current [`Block`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        // If the eternity is empty, we need to insert a new `Block` into a new `Epoch`, then insert
        //that `Epoch` into the eternity before we can insert an item into that `Block`
        let initialized = if self.inner.is_empty() {
            let mut tier = Tier::default();
            if tier.insert(Insert::Keep(Tier::default())).is_err()
                || self.inner.insert(Insert::Keep(tier)).is_err()
            {
                return Err(item);
            } else {
                true
            }
        } else {
            false
        };

        let result = self
            .inner
            .update(|focus| {
                if let Insert::Keep(focus) = focus {
                    focus
                        .update(|focus| {
                            if let Insert::Keep(focus) = focus {
                                focus.insert(item.map(Hash::of)).map_err(|_| item)
                            } else {
                                Err(item)
                            }
                        })
                        .unwrap_or(Err(item))
                } else {
                    Err(item)
                }
            })
            .unwrap_or(Err(item));

        if result.is_ok() {
            if initialized {
                self.epochs_witnessed += 1;
                self.blocks_witnessed += 1;
            }

            self.len += 1;
            self.items_witnessed += 1;
        }

        result
    }

    /// The total number of [`Fq`]s witnessed in every [`Block`] in every [`Epoch`] in this [`Eternity`].
    pub fn items_witnessed(&self) -> u16 {
        self.items_witnessed
    }

    /// The total number of [`Fq`]s or [`struct@Hash`]es represented in this [`Epoch`].
    ///
    /// This count includes those which were elided due to a partially filled [`Block`] or
    /// [`Epoch`], or summary root [`struct@Hash`] of a block or epoch being inserted.
    ///
    /// In other words, this is `2 ^ 32` times the number of epochs represented in this
    /// [`Eternity`], plus `4 ^ 8` times the number of blocks represented in this [`Eternity`],
    /// plus the number of items in the latest block.
    ///
    /// The maximum capacity of an [`Eternity`] is `2 ^ 48`, i.e. `4 ^ 8` epochs of `4 ^ 8` blocks
    /// of `4 ^ 8` items.
    pub fn len(&self) -> u64 {
        self.len
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
    pub fn hash(&self) -> Hash {
        self.inner.hash()
    }
}
