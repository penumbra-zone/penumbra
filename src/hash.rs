use crate::{internal::height::Zero, Commitment, Insert};

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
// TODO: replace this with `Fq`
pub struct Hash([u64; 4]);

impl<T: GetHash> From<&T> for Hash {
    fn from(item: &T) -> Self {
        item.hash()
    }
}

#[allow(unused)]
impl Hash {
    #[inline]
    pub(crate) fn padding() -> Hash {
        Hash(todo!("hash for padding"))
    }

    #[inline]
    pub(crate) fn commitment(commitment: &Commitment) -> Hash {
        Hash(todo!("hash commitment"))
    }

    #[inline]
    pub(crate) fn node(height: usize, a: Hash, b: Hash, c: Hash, d: Hash) -> Hash {
        Hash(todo!("hash node"))
    }

    /// Get the hashes of all the `HashOr<T>` in the array, hashing `T` as necessary.
    #[inline]
    pub(crate) fn hashes_of_all<T: GetHash, const N: usize>(full: [&Insert<T>; N]) -> [Hash; N] {
        full.map(|hash_or_t| match hash_or_t {
            Insert::Hash(hash) => *hash,
            Insert::Keep(t) => t.hash(),
        })
    }
}

impl GetHash for Hash {
    #[inline]
    fn hash(&self) -> Hash {
        *self
    }
}

impl<T: GetHash> GetHash for &T {
    #[inline]
    fn hash(&self) -> Hash {
        (**self).hash()
    }
}

impl<T: GetHash> GetHash for &mut T {
    #[inline]
    fn hash(&self) -> Hash {
        (**self).hash()
    }
}

impl crate::Height for Hash {
    type Height = Zero;
}

impl crate::Focus for Hash {
    type Complete = Self;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        Insert::Keep(self)
    }
}

impl crate::Complete for Hash {
    type Active = Self;
}
