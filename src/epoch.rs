use crate::*;

/// A sparse commitment tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536 [`Fq`]s
/// or their [`struct@Hash`]es.
///
/// This is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Epoch {
    pub(super) blocks_witnessed: u16,
    pub(super) items_witnessed: u16,
    pub(super) len: u32,
    pub(super) inner: Tier<Tier<Hash>>,
}

impl Height for Epoch {
    type Height = <Tier<Tier<Hash>> as Height>::Height;
}

impl Epoch {
    /// Create a new empty [`Epoch`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Block`] or its root [`struct@Hash`] all at once to the current [`Epoch`] of this
    /// [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Epoch`] if the
    /// [`Epoch`] is full.
    pub fn insert_block(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        let keep = block.is_keep();

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
            let block_start = self.len & (!(u16::MAX as u32));
            // The size of each block (2^16)
            let block_size = 1 << 16;
            // The new length is the start index of the *next* block plus the size of the one being added
            self.len = block_start + block_size + (len as u32);

            if keep {
                self.blocks_witnessed += 1;
            }
            self.items_witnessed += items_witnessed;
        }

        result
    }

    /// The number of [`Block`]s witnessed in this [`Epoch`].
    pub fn blocks_witnessed(&self) -> u16 {
        self.blocks_witnessed
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to the current [`Block`] of this [`Epoch`].
    ///
    /// # Errors
    ///
    /// Returns `Err(block)` containing the inserted block without adding it to the [`Epoch`] if:
    ///
    /// 1. the [`Epoch`] is full,
    /// 2. the current [`Block`] is full, or
    /// 3. the current [`Block`] was inserted as [`Insert::Hash`], which means that it cannot be
    /// modified after insertion.
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        let keep = item.is_keep();

        // If the epoch is empty, we need to insert a new `Block` before we can insert into that block
        let initialized = if self.inner.is_empty() {
            if self.inner.insert(Insert::Keep(Tier::default())).is_err() {
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
                    focus.insert(item.map(Hash::of)).map_err(|_| item)
                } else {
                    Err(item)
                }
            })
            .unwrap_or(Err(item));

        if result.is_ok() {
            if initialized {
                self.blocks_witnessed += 1;
            }
            if keep {
                self.items_witnessed += 1;
            }
            self.len += 1;
        }

        result
    }

    /// The total number of [`Fq`]s witnessed in every [`Block`] in this [`Epoch`].
    pub fn items_witnessed(&self) -> u16 {
        self.items_witnessed
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
        self.len
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
}
