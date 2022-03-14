//! Every structure in this crate can be hashed, and many use interior mutation to cache their
//! hashes lazily.
//!
//! This module defines the trait [`GetHash`] for these operations, as well as the [`struct@Hash`] type
//! used throughout.

use std::fmt::Debug;

use ark_ff::{fields::PrimeField, BigInteger256, Fp256, ToBytes};
use once_cell::sync::Lazy;
use poseidon377::Fq;

use crate::{
    internal::{height::Zero, path},
    AuthPath, Complete, ForgetOwned, Height, Insert, Item, Witness,
};

mod option_hash;
pub use option_hash::OptionHash;

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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
/// The hash of an individual item, tree root, or intermediate node. Use [`Insert::Hash`] with this
/// type when you want to insert something into the tree that you don't want to witness later.
pub struct Hash(Fq);

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut bytes = Vec::with_capacity(4 * 8);
        self.0.write(&mut bytes).unwrap();
        write!(f, "{}", hex::encode(&bytes[3 * 8 + 4..]))
    }
}

impl<T: GetHash> From<&T> for Hash {
    fn from(item: &T) -> Self {
        item.hash()
    }
}

/// The domain separator used for leaves in the tree, and used as a base index for the domain
/// separators of nodes in the tree (nodes get a domain separator of the form `DOMAIN_SEPARATOR +
/// HEIGHT`).
pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.tct").as_bytes()));

#[allow(unused)]
impl Hash {
    /// Hash an individual item to be inserted into the tree.
    #[inline]
    pub fn of(item: Fq) -> Hash {
        Hash(poseidon377::hash_1(&DOMAIN_SEPARATOR, item))
    }

    /// Get the underlying bytes for the hash
    pub(crate) fn into_bytes(self) -> [u64; 4] {
        self.0 .0 .0
    }

    /// Construct a hash from bytes directly without checking whether they are in range for [`Fq`].
    ///
    /// This should only be called when you know that the bytes are valid.
    pub(crate) fn from_bytes_unchecked(bytes: [u64; 4]) -> Hash {
        Self(Fp256::new(BigInteger256(bytes)))
    }

    #[inline]
    pub(crate) fn node(
        height: u8,
        Hash(a): Hash,
        Hash(b): Hash,
        Hash(c): Hash,
        Hash(d): Hash,
    ) -> Hash {
        let height = Fq::from_le_bytes_mod_order(&height.to_le_bytes());
        Hash(poseidon377::hash_4(
            &(*DOMAIN_SEPARATOR + height),
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

impl Height for Hash {
    type Height = Zero;
}

impl Complete for Hash {
    type Focus = Item;
}

impl Witness for Hash {
    type Item = Hash;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        if index.into() == 0 {
            Some((path::Leaf, *self))
        } else {
            None
        }
    }
}

impl ForgetOwned for Hash {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        if index.into() == 0 {
            (Insert::Hash(self), true)
        } else {
            (Insert::Keep(self), false)
        }
    }
}
