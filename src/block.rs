use crate::*;

/// A sparse commitment tree to witness up to 65,536 individual [`Fq`]s or their [`struct@Hash`]es.
///
/// This is one [`Block`] in an [`Epoch`], which is one [`Epoch`] in an [`Eternity`].
#[derive(Derivative, Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub(super) inner: Tier<Hash>,
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
    pub fn insert_item(&mut self, item: Insert<Fq>) -> Result<(), Insert<Fq>> {
        self.inner.insert(item.map(Hash::of)).map_err(|_| item)
    }

    /// The number of items witnessed in this [`Block`].
    pub fn items_witnessed(&self) -> u16 {
        self.inner.size()
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_eternity_size() {
        static_assertions::assert_eq_size!(Eternity, [u8; 80]);
    }
}
