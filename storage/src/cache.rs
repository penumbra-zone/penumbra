use std::{any::Any, cmp::Ordering, collections::BTreeMap, pin::Pin};

use anyhow::Result;
use async_stream::stream;
use futures::{Stream, StreamExt};
use tendermint::abci;

use crate::{future::CacheFuture, StateWrite};

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
        // One might ask, why does this exist separately from `apply_to`?  The
        // answer is that `apply_to` takes a `StateWrite`, so we'd have to have
        // `Cache: StateWrite`, and that implies `Cache: StateRead`, but the
        // `StateRead` trait assumes asynchronous access, and in any case, we
        // probably don't want to be reading directly from a `Cache` (?)
        self.unwritten_changes.extend(other.unwritten_changes);
        self.nonconsensus_changes.extend(other.nonconsensus_changes);
        self.ephemeral_objects.extend(other.ephemeral_objects);
        self.events.extend(other.events);
    }

    /// Consume this cache, applying its writes to the given state.
    pub fn apply_to<S: StateWrite>(self, mut state: S) {
        for (key, value) in self.unwritten_changes {
            if let Some(value) = value {
                state.put_raw(key, value);
            } else {
                state.delete(key);
            }
        }

        for (key, value) in self.nonconsensus_changes {
            if let Some(value) = value {
                state.nonconsensus_put_raw(key, value);
            } else {
                state.nonconsensus_delete(key);
            }
        }

        for (key, value) in self.ephemeral_objects {
            if let Some(value) = value {
                state.object_put(key, value);
            } else {
                state.object_delete(key);
            }
        }

        for event in self.events {
            state.record(event);
        }
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

    pub fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
        underlying: impl Stream<Item = Result<(String, Vec<u8>)>> + Send + Unpin + 'a,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>> {
        // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
        // until we reach the keys that no longer match the prefix.
        let unwritten_changes_iter = self
            .unwritten_changes
            .range(prefix.to_string()..)
            .take_while(move |(k, _)| (**k).starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()));

        merge_cache(unwritten_changes_iter, underlying).boxed()
    }

    pub fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
        underlying: impl Stream<Item = Result<String>> + Send + Unpin + 'a,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>> {
        // The implementation is similar to prefix_raw_with_cache.  In order to
        // reuse the merge_cache code, we use zero-size dummy values (), which lets
        // us correctly handle uncommitted deletions (which will be represented as
        // None rather than Some(())).

        // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
        // until we reach the keys that no longer match the prefix.
        let unwritten_changes_iter = self
            .unwritten_changes
            .range(prefix.to_string()..)
            .take_while(move |(k, _)| (**k).starts_with(prefix))
            // Do a little dance to turn &Some(bytes) into Some(()), and &None into None,
            // so we're only working with keys, not values, but we can reuse the merge_cache function.
            .map(|(k, v)| (k.clone(), v.as_ref().map(|_| ())));

        merge_cache(
            unwritten_changes_iter,
            // Tag all the underlying items with dummy values ()...
            underlying.map(move |r| r.map(move |k| (k, ()))),
        )
        // ...and then strip them back off again.
        .map(|r| r.map(|(k, ())| k))
        .boxed()
    }

    pub fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
        underlying: impl Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + Unpin + 'a,
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>> {
        // Range the nonconsensus_changes cache (sorted by key) starting with the keys matching the prefix,
        // until we reach the keys that no longer match the prefix.
        let nonconsensus_changes_iter = self
            .nonconsensus_changes
            .range(prefix.to_vec()..)
            .take_while(move |(k, _)| (**k).starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()));

        merge_cache(nonconsensus_changes_iter, underlying).boxed()
    }
}

/// Merge a RYW cache iterator with a backend storage stream to produce a new
/// Stream.
///
/// When two streams have the same key, the cache takes priority. `None` values
/// represent deletions and are skipped in the output stream.
fn merge_cache<'a, K, V>(
    cache: impl Iterator<Item = (K, Option<V>)> + Send + Sync + Unpin + 'a,
    storage: impl Stream<Item = Result<(K, V)>> + Send + Unpin + 'a,
) -> impl Stream<Item = Result<(K, V)>> + Send + 'a
where
    V: Send + Clone + Sync + 'a,
    K: Send + Clone + Sync + 'a,
    K: Ord,
{
    stream! {
        let mut cache = cache.peekable();
        let mut storage = storage.peekable();

        loop {
            match (cache.peek(), Pin::new(&mut storage).peek().await) {
                (Some(cached), Some(Ok(stored))) => {
                    // Cache takes priority.
                    // Compare based on key ordering
                    match cached.0.cmp(&stored.0) {
                        Ordering::Less => {
                            // unwrap() is safe because `peek()` succeeded
                            let (k, v) = cache.next().unwrap();
                            match v {
                                Some(v) => yield Ok((k.clone(), v.clone())),
                                // Skip keys deleted in the cache.
                                None => continue,
                            }
                        },
                        Ordering::Equal => {
                            // Advance the right-hand side since the keys matched, and
                            // the left takes precedence.
                            storage.next().await;
                            // unwrap() is safe because `peek()` succeeded
                            let (k, v) = cache.next().unwrap();
                            match v {
                                Some(v) => yield Ok((k.clone(), v.clone())),
                                // Skip keys deleted in the cache.
                                None => continue,
                            }
                        },
                        Ordering::Greater => {
                            // unwrap() is safe because `peek()` succeeded
                            yield storage.next().await.unwrap();
                        },
                    }
                }
                (_, Some(Err(_e))) => {
                    // If we have a storage error, we want to report it immediately.
                    // If `peek` errored, this is also guaranteed to error.
                    yield storage.next().await.unwrap();
                    break;
                }
                (Some(_cached), None) => {
                    // Exists only in cache
                    let (k, v) = cache.next().unwrap();
                    match v {
                        Some(v) => yield Ok((k.clone(), v.clone())),
                        // Skip keys deleted in the cache.
                        None => continue,
                    }
                }
                (None, Some(Ok(_stored))) => {
                    // Exists only in storage
                    yield storage.next().await.unwrap();
                }
                (None, None) => break,
            }
        }
    }
}
