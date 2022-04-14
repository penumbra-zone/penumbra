//! Every structure in this crate can be hashed, and many use interior mutation to cache their
//! hashes lazily.
//!
//! This module defines the trait [`GetHash`] for these operations, as well as the [`struct@Hash`] type
//! used throughout.

use std::{fmt::Debug, marker::PhantomData};

use ark_ff::{fields::PrimeField, BigInteger256, Fp256, ToBytes};
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use poseidon377::Fq;
use serde::{Deserialize, Serialize};

// #[cfg(not(feature = "fast_hash"))]
// use poseidon377::{hash_1, hash_4};

// #[cfg(feature = "fast_hash")]
// mod fast_hash;
// #[cfg(feature = "fast_hash")]
// use fast_hash::{hash_1, hash_4};

use crate::Commitment;

mod option_hash;
pub use option_hash::OptionHash;

/// A type which can be transformed into a [`struct@Hash`], either by retrieving a cached hash, computing a
/// hash for it, or some combination of both.
pub trait GetHash<Hasher> {
    /// Get the hash of this item.
    ///
    /// # Correctness
    ///
    /// This function must return the same hash for the same item. It is permissible to use internal
    /// mutability to cache hashes, but caching must ensure that the item cannot be mutated without
    /// recalculating the hash.
    fn hash(&self) -> Hash<Hasher>;

    /// Get the hash of this item, only if the hash is already cached and does not require
    /// recalculation.
    ///
    /// # Correctness
    ///
    /// It will not cause correctness issues to return a hash after recalculating it, but users of
    /// this function expect it to be reliably fast, so it may cause unexpected performance issues
    /// if this function performs any significant work.
    fn cached_hash(&self) -> Option<Hash<Hasher>>;
}

impl<T: GetHash<Hasher>, Hasher> GetHash<Hasher> for &T {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        (**self).hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        (**self).cached_hash()
    }
}

impl<T: GetHash<Hasher>, Hasher> GetHash<Hasher> for &mut T {
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        (**self).hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        (**self).cached_hash()
    }
}

pub trait Hasher {
    fn hash_1(domain_separator: &Fq, value: Fq) -> Fq;

    fn hash_4(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq)) -> Fq;
}

pub struct Poseidon377;

impl Hasher for Poseidon377 {
    #[inline]
    fn hash_1(domain_separator: &Fq, value: Fq) -> Fq {
        poseidon377::hash_1(domain_separator, value)
    }

    #[inline]
    fn hash_4(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq)) -> Fq {
        poseidon377::hash_4(domain_separator, value)
    }
}

pub struct Blake2b;

impl Hasher for Blake2b {
    fn hash_1(domain_separator: &Fq, value: Fq) -> Fq {
        let mut state = blake2b_simd::State::new();
        state.update(&domain_separator.to_bytes());
        state.update(&value.to_bytes());
        Fq::from_le_bytes_mod_order(state.finalize().as_bytes())
    }

    fn hash_4(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq)) -> Fq {
        let mut state = blake2b_simd::State::new();
        state.update(&domain_separator.to_bytes());
        state.update(&value.0.to_bytes());
        state.update(&value.1.to_bytes());
        state.update(&value.2.to_bytes());
        state.update(&value.3.to_bytes());
        Fq::from_le_bytes_mod_order(state.finalize().as_bytes())
    }
}

/// The hash of an individual item, tree root, or intermediate node.
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(
    Copy(bound = ""),
    Clone(bound = ""),
    Default(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = ""),
    Hash(bound = "")
)]
pub struct Hash<Hasher /*  = Poseidon377 */>(
    #[serde(with = "crate::serialize::fq")] Fq,
    PhantomData<Hasher>,
);

impl<Hasher> From<Hash<Hasher>> for Fq {
    #[inline]
    fn from(hash: Hash<Hasher>) -> Self {
        hash.0
    }
}

impl<Hasher> Debug for Hash<Hasher> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut bytes = Vec::with_capacity(4 * 8);
        self.0.write(&mut bytes).unwrap();
        write!(f, "{}", hex::encode(&bytes[3 * 8 + 4..]))
    }
}

/// The domain separator used for leaves in the tree, and used as a base index for the domain
/// separators of nodes in the tree (nodes get a domain separator of the form `DOMAIN_SEPARATOR +
/// HEIGHT`).
pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.tct").as_bytes()));

#[allow(unused)]
impl<H> Hash<H> {
    /// Create a hash from an arbitrary [`Fq`].
    pub(crate) fn new(fq: Fq) -> Self {
        Self(fq, PhantomData)
    }

    /// Get the underlying bytes for the hash
    pub(crate) fn into_bytes(self) -> [u64; 4] {
        self.0 .0 .0
    }

    /// Construct a hash from bytes directly without checking whether they are in range for [`Commitment`].
    ///
    /// This should only be called when you know that the bytes are valid.
    pub(crate) fn from_bytes_unchecked(bytes: [u64; 4]) -> Self {
        Self::new(Fp256::new(BigInteger256(bytes)))
    }

    /// Hash an individual item to be inserted into the tree.
    #[inline]
    pub fn of(item: Commitment) -> Self
    where
        H: Hasher,
    {
        Self::new(H::hash_1(&DOMAIN_SEPARATOR, item.into()))
    }

    /// Construct a hash for an internal node of the tree, given its height and the hashes of its
    /// four children.
    #[inline]
    pub fn node(
        height: u8,
        Self(a, _): Self,
        Self(b, _): Self,
        Self(c, _): Self,
        Self(d, _): Self,
    ) -> Self
    where
        H: Hasher,
    {
        let height = Fq::from_le_bytes_mod_order(&height.to_le_bytes());
        Self::new(H::hash_4(&(*DOMAIN_SEPARATOR + height), (a, b, c, d)))
    }
}

#[cfg(feature = "sqlx")]
mod sqlx_impls {
    use decaf377::{FieldExt, Fq};
    use sqlx::{Database, Decode, Encode, Postgres, Type};
    use thiserror::Error;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("expected exactly 32 bytes")]
    struct IncorrectLength;

    impl<'r, Hasher> Decode<'r, Postgres> for Hash<Hasher> {
        fn decode(
            value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let bytes: [u8; 32] = Vec::<u8>::decode(value)?
                .try_into()
                .map_err(|_| IncorrectLength)?;
            Ok(Hash::new(Fq::from_bytes(bytes)?))
        }
    }

    impl<'q, Hasher> Encode<'q, Postgres> for Hash<Hasher> {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            let bytes = self.0.to_bytes();
            (&bytes[..]).encode(buf)
        }
    }

    impl<Hasher> Type<Postgres> for Hash<Hasher> {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            <[u8] as Type<Postgres>>::type_info()
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
mod arbitrary {
    use std::marker::PhantomData;

    use super::Hash;

    impl<Hasher> proptest::arbitrary::Arbitrary for Hash<Hasher> {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            HashStrategy::default()
        }

        type Strategy = HashStrategy<Hasher>;
    }

    #[derive(Derivative)]
    #[derivative(
        Clone(bound = ""),
        Copy(bound = ""),
        Debug(bound = ""),
        PartialEq(bound = ""),
        Eq(bound = ""),
        Default(bound = "")
    )]
    pub struct HashStrategy<Hasher>(PhantomData<Hasher>);

    impl<Hasher> proptest::strategy::Strategy for HashStrategy<Hasher> {
        type Tree = proptest::strategy::Just<Hash<Hasher>>;

        type Value = Hash<Hasher>;

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
            Ok(proptest::strategy::Just(Hash::new(decaf377::Fq::new(
                ark_ff::BigInteger256(parts),
            ))))
        }
    }
}
