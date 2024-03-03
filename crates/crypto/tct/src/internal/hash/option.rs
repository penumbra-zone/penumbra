use decaf377::Fq;
use std::fmt::Debug;

use crate::prelude::*;

/// A representation of `Option<Hash>` without the tag bytes required by `Option`, because we
/// know that no valid [`struct@Hash`] will be equal to `[u64::MAX; 4]`, since the modulus for
/// [`Commitment`](crate::Commitment) is too small.
///
/// This type is inter-convertible via [`From`] and [`Into`] with `Option<Hash>`, and that is
/// its only purpose.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Option<Hash>", into = "Option<Hash>")]
pub struct OptionHash {
    inner: [u64; 4],
}

impl Debug for OptionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Option<Hash>>::from(*self).fmt(f)
    }
}

impl Default for OptionHash {
    fn default() -> Self {
        Self {
            inner: [u64::MAX; 4],
        }
    }
}

impl From<Option<Hash>> for OptionHash {
    fn from(hash: Option<Hash>) -> Self {
        match hash {
            Some(hash) => Self {
                inner: hash.0.to_le_limbs(),
            },
            None => Self {
                // This sentinel value is not a valid `Fq` because it's bigger than the modulus,
                // which means that it will never occur otherwise
                inner: [u64::MAX; 4],
            },
        }
    }
}

impl From<OptionHash> for Option<Hash> {
    fn from(hash: OptionHash) -> Self {
        if hash.inner == [u64::MAX; 4] {
            None
        } else {
            // We're directly constructing the hash here by coercing the bytes into the right type,
            // but this is safe because we know that the bytes are a real `Fq` and not the sentinel
            // value we just checked for
            Some(Hash::new(Fq::from_le_limbs(hash.inner)))
        }
    }
}
