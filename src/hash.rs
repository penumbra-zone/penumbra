use crate::{height::Z, Commitment};

/// A type which can be transformed into a [`Hash`], either by retrieving a cached hash, computing a
/// hash for it, or some combination of both.
pub trait GetHash {
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
    #[inline]
    pub(crate) fn padding() -> Hash {
        Hash
    }

    #[inline]
    pub(crate) fn commitment(commitment: &Commitment) -> Hash {
        Hash
    }

    #[inline]
    pub(crate) fn node(height: usize, a: Hash, b: Hash, c: Hash, d: Hash) -> Hash {
        Hash
    }

    /// Get the hashes of all the `HashOr<T>` in the array, hashing `T` as necessary.
    #[inline]
    pub(crate) fn hashes_of_all<T: GetHash, const N: usize>(full: [&HashOr<T>; N]) -> [Hash; N] {
        full.map(|hash_or_t| match hash_or_t {
            HashOr::Hash(hash) => *hash,
            HashOr::Item(t) => t.hash(),
        })
    }
}

impl GetHash for Hash {
    #[inline]
    fn hash(&self) -> Hash {
        *self
    }
}

impl crate::Height for Hash {
    type Height = Z;
}

impl crate::Focus for Hash {
    type Complete = Self;

    #[inline]
    fn finalize(self) -> HashOr<Self::Complete> {
        HashOr::Item(self)
    }
}

impl crate::Complete for Hash {
    type Active = Self;
}

/// Either a hash or some item, or the item itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashOr<T> {
    /// The hash of an item.
    Hash(Hash),
    /// The item itself.
    Item(T),
}
