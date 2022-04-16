//! A thread-safe cache intended hold lazily evaluated hashes.

use std::cell::Cell;

use parking_lot::Once;

use crate::{internal::hash::OptionHash, Hash};

/// An `RwLock`-based cache for things isomorphic to `Option<Hash>`.
///
/// In our use case, `T` is `OptionHash`, but this is written generically for ease of
/// implementation.
#[derive(Debug, Default)]
pub struct CachedHash {
    cell: Cell<OptionHash>,
    once: Once,
}

// This is safe because the restricted API of `CachedHash` only sets the internal hash *at most
// once*, from only one thread, using a `Once` to ensure multiple threads do not race on setting the
// hash. The only way to do interior mutability is via `set_if_empty`, which mutates the inner
// `Cell` exactly once.
unsafe impl Sync for CachedHash {}

// Because `Once` cannot be cloned, we need to clone the `CachedHash` manually, creating a new
// `Once` for the clone. This could mean that some repeated effort is performed, except that if the
// cache has already been populated, we don't bother to call the closure that recomputes the hash,
// which means we aren't relying on the `Once` synchronization guard to prevent repeated heavy
// computation.
impl Clone for CachedHash {
    fn clone(&self) -> Self {
        CachedHash {
            once: Once::new(),
            cell: self.cell.clone(),
        }
    }
}

impl CachedHash {
    /// Get the value in the cache, given that the value is `Copy`.
    pub fn get(&self) -> Option<Hash> {
        self.cell.get().into()
    }

    /// If the cache is empty, set its value using the closure, then return its contents regardless.
    pub fn set_if_empty(&self, new: impl FnOnce() -> Hash) -> Hash {
        self.once.call_once(|| {
            if <Option<Hash>>::from(self.cell.get()).is_none() {
                // IMPORTANT: here is the **ONLY** place where we mutate the inner `Cell`, and it's
                // guarded by the `Once`:
                self.cell.set(Some(new()).into());
            }
        });
        self.get().unwrap()
    }
}
