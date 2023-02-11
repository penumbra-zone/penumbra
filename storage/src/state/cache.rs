use std::{any::Any, collections::BTreeMap};

use tendermint::abci;

use crate::future::CacheFuture;

/// A cache of changes to the state of the blockchain.
///
/// Used internally by `State` and `StateTransaction`.
#[derive(Default, Debug)]
pub struct Cache {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonconsensus storage).
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Unwritten changes to the object store.  A `None` value means a deletion.
    pub(crate) ephemeral_objects: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    /// A list of ABCI events that occurred while building this set of state changes.
    pub(crate) events: Vec<abci::Event>,
}

impl Cache {
    /// Merge the given cache with this one, taking its writes in place of ours.
    pub fn merge(&mut self, other: Cache) {
        self.unwritten_changes.extend(other.unwritten_changes);
        self.nonconsensus_changes.extend(other.nonconsensus_changes);
        self.ephemeral_objects.extend(other.ephemeral_objects);
        self.events.extend(other.events);
    }

    /// Returns `true` if there are cached writes on top of the snapshot, and `false` otherwise.
    pub fn is_dirty(&self) -> bool {
        !(self.unwritten_changes.is_empty()
            && self.nonconsensus_changes.is_empty()
            && self.ephemeral_objects.is_empty())
    }

    /// Use this cache to get a value by key, or else fetch a cache miss asynchronously.
    ///
    /// Taking a closure that produces the future means we can avoid creating it if the key
    /// is present in the cache.
    pub fn get_raw_or_else<Fn, Miss>(&self, key: &str, f: Fn) -> CacheFuture<Miss>
    where
        Fn: FnOnce() -> Miss,
    {
        match self.unwritten_changes.get(key) {
            // If the key is present in the cache, return its value synchronously.
            Some(v) => CacheFuture::hit(v.clone()),
            // Otherwise, prepare to fetch the value asynchronously.
            None => CacheFuture::miss(f()),
        }
    }

    /// Use this cache to get a value by key, or else fetch a cache miss asynchronously.
    ///
    /// Taking a closure that produces the future means we can avoid creating it if the key
    /// is present in the cache.
    pub fn nonconsensus_get_raw_or_else<Fn, Miss>(&self, key: &[u8], f: Fn) -> CacheFuture<Miss>
    where
        Fn: FnOnce() -> Miss,
    {
        match self.nonconsensus_changes.get(key) {
            // If the key is present in the cache, return its value synchronously.
            Some(v) => CacheFuture::hit(v.clone()),
            // Otherwise, prepare to fetch the value asynchronously.
            None => CacheFuture::miss(f()),
        }
    }
}
