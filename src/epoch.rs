use crate::*;

/// A sparse commitment tree to witness up to 65,536 [`Block`]s, each witnessing up to 65,536 [`Fq`]s
/// or their [`struct@Hash`]es.
///
/// This is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Epoch {
    pub(super) inner: Tier<Tier<Item>>,
}

impl Height for Epoch {
    type Height = <Tier<Tier<Item>> as Height>::Height;
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
    pub fn insert(&mut self, block: Insert<Block>) -> Result<(), Insert<Block>> {
        self.inner
            .insert(block.map(|block| block.inner))
            .map_err(|block| block.map(|inner| Block { inner }))
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
    pub fn witness(&self, index: usize) -> Option<Proof<Epoch>> {
        let (auth_path, leaf) = self.inner.witness(index)?;
        Some(Proof {
            index,
            auth_path,
            leaf,
        })
    }
}
