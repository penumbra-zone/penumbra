//! A thread-safe cache intended hold lazily evaluated hashes.

use parking_lot::RwLock;

/// An `RwLock`-based cache for things isomorphic to `Option<S>` for some `S`.
///
/// In our use case, `T` is `OptionHash` and `S` is always `Hash`, but this is written generically
/// for ease of implementation.
#[derive(Debug, Default)]
pub struct Cache<T: Copy>(RwLock<T>);

impl<T: Copy> Clone for Cache<T> {
    fn clone(&self) -> Self {
        Cache(RwLock::new((*self.0.read()).clone()))
    }
}

impl<T: Copy> Cache<T> {
    /// Make a new [`Cache`] with the given value.
    pub fn new<S>(value: Option<S>) -> Self
    where
        T: From<Option<S>>,
    {
        Cache(RwLock::new(value.into()))
    }

    /// Get the value in the cache, given that the value is `Copy`.
    pub fn get<S>(&self) -> Option<S>
    where
        T: Into<Option<S>>,
    {
        (*self.0.read()).into()
    }

    /// Set the cached value unconditionally.
    pub fn set<S>(&self, value: Option<S>)
    where
        T: From<Option<S>>,
    {
        *self.0.write() = value.into();
    }

    /// If the cache is empty, set its value using the closure, then return its contents regardless.
    pub fn set_if_empty<S: Copy>(&self, new: impl FnOnce() -> S) -> S
    where
        T: Into<Option<S>> + From<Option<S>>,
    {
        // Optimistically try for a read lock, and bail if the contained thing is already `Some`
        if let Some(result) = (*self.0.read()).into() {
            return result;
        }

        // If not, get a write lock
        let mut lock = self.0.write();

        // Now things might have changed in between releasing the read lock and acquiring the
        // write lock, so check again and bail if the contained thing is already `Some`
        if let Some(result) = (*self.0.read()).into() {
            return result;
        }

        // Otherwise, finally run the closure to generate a new value and result
        let new = new();
        *lock = Some(new).into();
        new
    }
}
