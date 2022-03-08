//! Every structure in this crate can be hashed, and many use interior mutation to cache their
//! hashes lazily.
//!
//! This module defines the trait [`GetHash`] for these operations, as well as the [`struct@Hash`] type
//! used throughout.

use ark_ff::fields::PrimeField;
use once_cell::sync::Lazy;
use poseidon377::Fq;

use crate::{internal::height::Zero, Insert};

/// A type which can be transformed into a [`struct@Hash`], either by retrieving a cached hash, computing a
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

    /// Get the hash of this item, only if the hash is already cached and does not require
    /// recalculation.
    ///
    /// # Correctness
    ///
    /// It will not cause correctness issues to return a hash after recalculating it, but users of
    /// this function expect it to be reliably fast, so it may cause unexpected performance issues
    /// if this function performs any significant work.
    fn cached_hash(&self) -> Option<Hash>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
/// The hash of an individual item, to be used when inserting into a tree.
///
/// Like [`Item`](crate::Item), [`struct@Hash`] itself implements [`Focus`](crate::Focus) and thus can be
/// used as the item of a tree, if it is not desired to store commitments at the leaves.
pub struct Hash(Fq);

impl<T: GetHash> From<&T> for Hash {
    fn from(item: &T) -> Self {
        item.hash()
    }
}

/// The domain separator used for items in the tree.
pub static ITEM_DOMAIN_SEPARATOR: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.tct.item").as_bytes())
});

/// The base domain separator used for nodes in the tree (the height of the node is added to this to
/// differentiate nodes at different heights).
pub static NODE_DOMAIN_SEPARATOR: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.tct.node").as_bytes())
});

#[allow(unused)]
impl Hash {
    /// Hash an individual item to be inserted into the tree.
    #[inline]
    pub fn item(item: Fq) -> Hash {
        Hash(poseidon377::hash_1(&ITEM_DOMAIN_SEPARATOR, item))
    }

    #[inline]
    pub(crate) fn node(
        height: usize,
        Hash(a): Hash,
        Hash(b): Hash,
        Hash(c): Hash,
        Hash(d): Hash,
    ) -> Hash {
        let height = Fq::from_le_bytes_mod_order(&height.to_le_bytes());
        Hash(poseidon377::hash_4(
            &(*NODE_DOMAIN_SEPARATOR + height),
            (a, b, c, d),
        ))
    }
}

impl GetHash for Hash {
    #[inline]
    fn hash(&self) -> Hash {
        *self
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(*self)
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
    type Focus = Self;
}
