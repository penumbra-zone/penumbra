use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::Hash;

/// A representation of `Option<Hash>` without the tag bytes required by `Option`, because we
/// know that no valid [`struct@Hash`] will be equal to `[u64::MAX; 4]`, since the modulus for
/// [`Commitment`](crate::Commitment) is too small.
///
/// This type is inter-convertible via [`From`] and [`Into`] with `Option<Hash>`, and that is
/// its only purpose.
#[derive(Derivative, Serialize, Deserialize)]
#[serde(from = "Option<Hash>", into = "Option<Hash>")]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct OptionHash<Hasher> {
    inner: [u64; 4],
    #[derivative(Debug = "ignore")]
    hasher: PhantomData<Hasher>,
}

impl<Hasher> Default for OptionHash<Hasher> {
    fn default() -> Self {
        Self {
            inner: [u64::MAX; 4],
            hasher: PhantomData,
        }
    }
}

impl<Hasher> From<Option<Hash<Hasher>>> for OptionHash<Hasher> {
    fn from(hash: Option<Hash<Hasher>>) -> Self {
        match hash {
            Some(hash) => Self {
                inner: hash.into_bytes(),
                hasher: PhantomData,
            },
            None => Self {
                inner: [u64::MAX; 4],
                hasher: PhantomData,
            },
        }
    }
}

impl<Hasher> From<OptionHash<Hasher>> for Option<Hash<Hasher>> {
    fn from(hash: OptionHash<Hasher>) -> Self {
        if hash.inner == [u64::MAX; 4] {
            None
        } else {
            Some(Hash::from_bytes_unchecked(hash.inner))
        }
    }
}
