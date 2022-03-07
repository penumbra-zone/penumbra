use crate::Commitment;

/// A type which can be transformed into a [`Hash`], either by retrieving a cached hash, computing a
/// hash for it, or some combination of both.
pub(crate) trait GetHash {
    /// Get the hash of this item.
    ///
    /// # Correctness
    ///
    /// This function must return the same hash for the same item. It is permissible to use internal
    /// mutability to cache hashes, but caching must ensure that the item cannot be mutated without
    /// recalculating the hash.
    fn hash(&self) -> Hash;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hash;

#[allow(unused)]
impl Hash {
    pub(crate) fn padding() -> Hash {
        Hash
    }

    pub(crate) fn commitment(height: usize, commitment: &Commitment) -> Hash {
        Hash
    }

    pub(crate) fn node(height: usize, a: Hash, b: Hash, c: Hash, d: Hash) -> Hash {
        Hash
    }
}
