use crate::*;

/// A sparse commitment tree to witness up to 65,536 [`Epoch`]s, each witnessing up to 65,536
/// [`Block`]s, each witnessing up to 65,536 [`Fq`]s or their [`struct@Hash`]es.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Eternity {
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
    pub fn insert(&mut self, epoch: Insert<Epoch>) -> Result<(), Insert<Epoch>> {
        self.inner
            .insert(epoch.map(|epoch| epoch.inner))
            .map_err(|epoch| epoch.map(|inner| Epoch { inner }))
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
    pub fn hash(&self) -> Hash {
        self.inner.hash()
    }

    /// Get a [`Proof`] of inclusion for the item at this index in the eternity.
    ///
    /// If the index is not witnessed in this eternity, return `None`.
    pub fn witness(&self, index: usize) -> Option<Proof<Eternity>> {
        let (auth_path, leaf) = self.inner.witness(index)?;
        Some(Proof {
            index,
            auth_path,
            leaf,
        })
    }
}
