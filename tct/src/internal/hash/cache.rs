//! A thread-safe cache intended hold lazily evaluated hashes.

use std::fmt::Debug;

use parking_lot::Mutex;

use crate::{internal::hash::OptionHash, Hash};

/// An `Mutex`-based cache for hashes, to prevent repeated computation.
#[derive(Default, Derivative)]
pub struct CachedHash {
    mutex: Mutex<OptionHash>,
}

impl Debug for CachedHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(hash) = <Option<Hash>>::from(*self.mutex.lock()) {
            write!(f, "{hash:?}")
        } else {
            write!(f, "_")
        }
    }
}

impl Clone for CachedHash {
    fn clone(&self) -> Self {
        Self {
            mutex: Mutex::new(*self.mutex.lock()),
        }
    }
}

impl CachedHash {
    /// Get the cached hash, or return `None` if it is not yet set.
    pub fn get(&self) -> Option<Hash> {
        (*self.mutex.lock()).into()
    }

    /// If the cache is empty, set its value using the closure, then return its contents regardless.
    pub fn set_if_empty(&self, new: impl FnOnce() -> Hash) -> Hash {
        let mut guard = self.mutex.lock();
        if <Option<Hash>>::from(*guard).is_none() {
            *guard = OptionHash::from(Some(new()));
        }
        Option::from(*guard).unwrap()
    }
}
