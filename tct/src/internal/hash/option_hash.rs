use serde::{Deserialize, Serialize};

use crate::Hash;

/// A representation of `Option<Hash>` without the tag bytes required by `Option`, because we
/// know that no valid [`struct@Hash`] will be equal to `[u64::MAX; 4]`, since the modulus for
/// [`Commitment`](crate::Commitment) is too small.
///
/// This type is inter-convertible via [`From`] and [`Into`] with `Option<Hash>`, and that is
/// its only purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Option<Hash>", into = "Option<Hash>")]
pub struct OptionHash {
    inner: [u64; 4],
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
                inner: hash.into_bytes(),
            },
            None => Self {
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
            Some(Hash::from_bytes_unchecked(hash.inner))
        }
    }
}
