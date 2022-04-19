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
// `Once` for the clone. To prevent repeated computation, we pre-call the `Once` in the clone if the
// `Once` in `self` has already been completed.
impl Clone for CachedHash {
    fn clone(&self) -> Self {
        let once = Once::new();

        // If, at the time of cloning, the `Once` has already completed, then we can "mark" the
        // newly created `Once` as also completed, so that calling `set_if_empty` on the cloned
        // `CachedHash` will immediately succeed, rather than running the closure.
        //
        // If on the other hand the state is in-progress, the value of the `Cell` is not yet
        // meaningful, because it hasn't been set yet, so we need to **not** mark the cloned
        // `CachedHash` as completed, which will mean that there may be repeated computation if
        // one thread clones the `CachedHash` during the execution of `set_if_empty`.
        let cell = if self.once.state().done() {
            once.call_once(|| {});
            self.cell.clone() // Only clone the internal cell if its value is meaningful
        } else {
            Cell::new(Hash::default())
        };

        CachedHash { once, cell }
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
