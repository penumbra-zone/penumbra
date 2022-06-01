//! The core [`Hash`](struct@Hash) type, which is used internally to represent hashes, the
//! [`GetHash`] trait for computing and caching hashes of things, and the [`CachedHash`] type, which
//! is used internally for lazy evaluation of hashes.

use std::fmt::Debug;

use ark_ff::{fields::PrimeField, BigInteger256, Fp256, One, ToBytes, Zero};
use once_cell::sync::Lazy;
use poseidon377::{hash_1, hash_4, Fq};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

mod cache;
mod option;
pub use {cache::CachedHash, option::OptionHash};

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

    /// If there is a hash cached, clear the cache.
    ///
    /// By default, this does nothing. Override this if there is a cache.
    fn clear_cached_hash(&self) {}
}

impl<T: GetHash> GetHash for &T {
    #[inline]
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }
}

impl<T: GetHash> GetHash for &mut T {
    #[inline]
    fn hash(&self) -> Hash {
        (**self).hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        (**self).cached_hash()
    }
}

/// The hash of an individual item, tree root, or intermediate node.
#[derive(Clone, Copy, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize)]
pub struct Hash(#[serde(with = "crate::serialize::fq")] Fq);

impl From<Hash> for Fq {
    #[inline]
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut bytes = Vec::with_capacity(4 * 8);
        self.0.write(&mut bytes).unwrap();
        write!(f, "{}", hex::encode(&bytes))
    }
}

/// The domain separator used for leaves in the tree, and used as a base index for the domain
/// separators of nodes in the tree (nodes get a domain separator of the form `DOMAIN_SEPARATOR +
/// HEIGHT`).
pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.tct").as_bytes()));

#[allow(unused)]
impl Hash {
    /// Create a hash from an arbitrary [`Fq`].
    pub(crate) fn new(fq: Fq) -> Self {
        Self(fq)
    }

    /// Get the underlying bytes for the hash
    pub(crate) fn into_bytes(self) -> [u64; 4] {
        self.0 .0 .0
    }

    /// Construct a hash from bytes directly without checking whether they are in range for [`Commitment`].
    ///
    /// This should only be called when you know that the bytes are valid.
    pub(crate) fn from_bytes_unchecked(bytes: [u64; 4]) -> Hash {
        Self(Fp256::new(BigInteger256(bytes)))
    }

    /// The zero hash, used for padding of frontier nodes.
    pub(crate) fn zero() -> Hash {
        Self(Fq::zero())
    }

    /// The one hash, used for padding of complete nodes.
    pub(crate) fn one() -> Hash {
        Self(Fq::one())
    }

    /// Hash an individual item to be inserted into the tree.
    #[inline]
    pub fn of(item: Commitment) -> Hash {
        Self(hash_1(&DOMAIN_SEPARATOR, item.0))
    }

    /// Construct a hash for an internal node of the tree, given its height and the hashes of its
    /// four children.
    #[inline]
    pub fn node(height: u8, Hash(a): Hash, Hash(b): Hash, Hash(c): Hash, Hash(d): Hash) -> Hash {
        let height = Fq::from_le_bytes_mod_order(&height.to_le_bytes());
        Hash(hash_4(&(*DOMAIN_SEPARATOR + height), (a, b, c, d)))
    }
}

/// A version tracking when a particular piece of the tree was explicitly forgotten.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    std::hash::Hash,
    Serialize,
    Deserialize,
    Default,
)]
pub struct Forgotten(u64);

impl Forgotten {
    /// Get the next forgotten-version after this one.
    pub fn next(&self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("forgotten should never overflow"),
        )
    }
}

impl From<Forgotten> for u64 {
    fn from(forgotten: Forgotten) -> Self {
        forgotten.0
    }
}

#[cfg(any(test, feature = "arbitrary"))]
mod arbitrary {
    use super::Hash;

    impl proptest::arbitrary::Arbitrary for Hash {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            HashStrategy
        }

        type Strategy = HashStrategy;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct HashStrategy;

    impl proptest::strategy::Strategy for HashStrategy {
        type Tree = proptest::strategy::Just<Hash>;

        type Value = Hash;

        fn new_tree(
            &self,
            runner: &mut proptest::test_runner::TestRunner,
        ) -> proptest::strategy::NewTree<Self> {
            use proptest::prelude::RngCore;
            let rng = runner.rng();
            let parts = [
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
                rng.next_u64(),
            ];
            Ok(proptest::strategy::Just(Hash(decaf377::Fq::new(
                ark_ff::BigInteger256(parts),
            ))))
        }
    }
}
