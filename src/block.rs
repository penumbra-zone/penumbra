use crate::*;

/// A sparse commitment tree to witness up to 65,536 individual [`Fq`]s or their [`struct@Hash`]es.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub(super) inner: Tier<Item>,
}

impl Height for Block {
    type Height = <Tier<Item> as Height>::Height;
}

impl Block {
    /// Create a new empty [`Block`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new [`Fq`] or its [`struct@Hash`] to this [`Block`].
    ///
    /// # Errors
    ///
    /// Returns `Err(item)` containing the inserted item without adding it to the [`Block`] if the
    /// block is full.
    pub fn insert(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.inner.insert(item.map(Item::new)).map_err(|_| item)
    }

    /// The total number of [`Fq`]s or [`struct@Hash`]es represented in this [`Block`].
    pub fn len(&self) -> u16 {
        self.inner.len()
    }

    /// Check whether this [`Block`] is empty.
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

    /// Get a [`Proof`] of inclusion for the item at this index in the block.
    ///
    /// If the index is not witnessed in this block, return `None`.
    pub fn witness(&self, index: usize) -> Option<Proof<Block>> {
        let (auth_path, leaf) = self.inner.witness(index)?;
        Some(Proof {
            index,
            auth_path,
            leaf,
        })
    }
}
