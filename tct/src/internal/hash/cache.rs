//! A thread-safe cache intended hold lazily evaluated hashes.

use std::{cell::Cell, fmt::Debug};

use parking_lot::Once;

use crate::Hash;

/// A write-once cache for `Hash`es, allowing lazy evaluation of hashes inside the tree.
#[derive(Default)]
pub struct CachedHash {
    /// A cell containing a cached hash value, which may not yet be set.
    ///
    /// IMPORTANT: It is **unsafe** to **read** or **write** this cell without synchronizing on the
    /// `Once` within this struct.
    cell: Cell<Hash>,
    /// A synchronization variable which we will use to ensure that it is set at most once, ever, in
    /// the lifetime of this `CachedHash` (thus allowing this struct to be marked `Sync`), and that
    /// the value of the `Cell` is only ever observed after it has been set.
    once: Once,
}

// This is safe because the restricted API of `CachedHash` only sets the internal hash *at most
// once*, from only one thread, using a `Once` to ensure multiple threads do not race on setting the
// hash. The only way to do interior mutability is via `set_if_empty`, which mutates the inner
// `Cell` exactly once. Additionally, the value of the inner `Cell` cannot be observed until after
// it has been fully set, which means it is impossible to observe partial writes, regardless of
// whether this would be possible in practice on any supported architecture.
unsafe impl Sync for CachedHash {}

// Because `Once` cannot be cloned, we need to clone the `CachedHash` manually, creating a new
// `Once` for the clone. This could mean that some repeated effort is performed, except that if the
// cache has already been populated, we don't bother to call the closure that recomputes the hash,
// which means we aren't relying on the `Once` synchronization guard to prevent repeated heavy
// computation.
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
        if self.once.state().done() {
            once.call_once(|| {});
        }

        CachedHash {
            once,
            cell: self.cell.clone(),
        }
    }
}

// A derived `Debug` implementation won't do, because the generated code would access the `Cell`
// without going through the `Once` synchronization barrier. Our manual implementation, in addition
// to ensuring that torn reads cannot be observed (by using the safe method `get`), gives a succinct
// representation of the cached hash, using `_` to denote a hash that hasn't yet been populated.
impl Debug for CachedHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(hash) = self.get() {
            write!(f, "{:?}", hash)
        } else {
            write!(f, "_")
        }
    }
}

impl CachedHash {
    /// Get the cached hash, if any has yet been set.
    pub fn get(&self) -> Option<Hash> {
        // IMPORTANT: we prevent ourselves from observing partial writes to the hash state by only
        // reading the contents of the cell if the `Once` has executed; otherwise, we just return
        // `None` without examining the `Cell` at all.
        //
        // Even this conservative approach does not allow accidental duplicate computation of
        // hashes, because even if a thread observes via `get` that a `CachedHash` is currently
        // `None`, its only way to set the hash is to use `set_if_empty`, which will not duplicate
        // computation if another thread is already in-progress computing the hash.
        //
        // A consequence of this very limited API is that it is impossible to observe the initial
        // value given to the `Hash` stored in the `Cell`: we only ever observe it once it has been
        // overwritten with a stored value. The initial value is `Hash::default()` because the only
        // way to construct a `CachedHash` is by the derived `CachedHash::default()`, but this value
        // is a convenience, not a meaningful choice.
        if self.once.state().done() {
            Some(self.cell.get())
        } else {
            None
        }
    }

    /// If the cache is empty, set its value using the closure, then return its contents regardless.
    pub fn set_if_empty(&self, new: impl FnOnce() -> Hash) -> Hash {
        // IMPORTANT: here is the **ONLY** place where we mutate the inner `Cell`, and it's
        // guarded by the `Once`:
        self.once.call_once(|| self.cell.set(new()));
        // Since we know that *some* initialization closure (not necessarily the one in this
        // thread!) has run, we can safely get the hash that is now in the cell and return it:
        self.cell.get()
    }
}
