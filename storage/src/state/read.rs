use std::{any::Any, cmp::Ordering, collections::BTreeMap, future::Future, pin::Pin};

use anyhow::Result;

use async_stream::stream;
use futures::{Stream, StreamExt};

/// Read access to chain state.
pub trait StateRead: Send + Sync {
    type GetRawFut: Future<Output = Result<Option<Vec<u8>>>> + Send + 'static;

    /// Gets a value from the verifiable key-value store as raw bytes.
    ///
    /// Users should generally prefer to use `get` or `get_proto` from an extension trait.
    fn get_raw(&self, key: &str) -> Self::GetRawFut;

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use `prefix` or `prefix_proto` from an extension trait.
    #[allow(clippy::type_complexity)]
    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>>;

    /// Retrieve all keys (but not values) matching a prefix from the verifiable key-value store.
    fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>>;

    /// Gets a byte value from the non-verifiable key-value store.
    ///
    /// This is intended for application-specific indexes of the verifiable
    /// consensus state, rather than for use as a primary data storage method.
    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut;

    /// Retrieve all values for keys matching a prefix from the non-verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use wrapper methods in an extension trait.
    #[allow(clippy::type_complexity)]
    fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>>;

    /// Gets an object from the ephemeral key-object store.
    ///
    /// This is intended to allow application components to build up batched
    /// data transactionally, ensuring that a transaction's contributions to
    /// some batched data are only included if the entire transaction executed
    /// successfully.  This data is not persisted to the `Storage` during
    /// `commit`.
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if a value of type `T` was present at `key`.
    /// - `None` if `key` was not present, or if `key` was present but the value was not of type `T`.
    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T>;
}

// Merge a RYW cache iterator with a backend storage stream to produce a new Stream,
// preferring results from the cache when keys are equal.
fn merge_cache<'a, K, V>(
    cache: impl Iterator<Item = (K, V)> + Send + Sync + Unpin + 'a,
    storage: impl Stream<Item = Result<(K, V)>> + Send + Unpin + 'a,
) -> impl Stream<Item = Result<(K, V)>> + Send + Unpin + 'a
where
    V: Send + Clone + Sync + 'a,
    K: Send + Clone + Sync + 'a,
    K: Ord,
{
    Box::pin(stream! {
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
                            yield Ok((k.clone(), v.clone()));
                        },
                        Ordering::Equal => {
                            // Advance the right-hand side since the keys matched, and
                            // the left takes precedence.
                            storage.next().await;
                            // unwrap() is safe because `peek()` succeeded
                            let (k, v) = cache.next().unwrap();
                            yield Ok((k.clone(), v.clone()));
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
                    yield Ok((k.clone(), v.clone()));
                }
                (None, Some(Ok(_stored))) => {
                    // Exists only in storage
                    yield storage.next().await.unwrap();
                }
                (None, None) => break,
            }
        }
    })
}

#[allow(clippy::type_complexity)]
pub(crate) fn prefix_raw_with_cache<'a>(
    sr: &'a impl StateRead,
    cache: &'a BTreeMap<String, Option<Vec<u8>>>,
    prefix: &'a str,
) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>> {
    // Interleave the unwritten_changes cache with the snapshot.
    let state_stream = sr
        .prefix_raw(prefix)
        .map(move |r| r.map(move |(k, v)| (k, Some(v))));

    // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
    // until we reach the keys that no longer match the prefix.
    let unwritten_changes_iter = cache
        .range(prefix.to_string()..)
        .take_while(move |(k, _)| (**k).starts_with(prefix))
        .map(|(k, v)| (k.clone(), v.clone()));

    // Merge the cache iterator and state stream into a single stream.
    let merged = merge_cache(unwritten_changes_iter, state_stream);

    // Skip all the `None` values, as they were deleted.
    let merged =
        merged.filter_map(|r| async { r.map(|(k, v)| v.map(move |v| (k, v))).transpose() });

    Box::pin(merged)
}

pub(crate) fn prefix_keys_with_cache<'a>(
    sr: &'a impl StateRead,
    cache: &'a BTreeMap<String, Option<Vec<u8>>>,
    prefix: &'a str,
) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>> {
    // The implementation is similar to prefix_raw_with_cache.  In order to
    // reuse the merge_cache code, we use zero-size dummy values (), which lets
    // us correctly handle uncommitted deletions (which will be represented as
    // None rather than Some(())).

    let state_stream = sr
        .prefix_keys(prefix)
        .map(move |r| r.map(move |k| (k, Some(()))));

    // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
    // until we reach the keys that no longer match the prefix.
    let unwritten_changes_iter = cache
        .range(prefix.to_string()..)
        .take_while(move |(k, _)| (**k).starts_with(prefix))
        // Do a little dance to turn &Some(bytes) into Some(()), and &None into None,
        // so we're only working with keys, not values, but we can reuse the merge_cache function.
        .map(|(k, v)| (k.clone(), v.as_ref().map(|_| ())));

    // Merge the unwritten_changes cache with the snapshot.
    let merged = merge_cache(unwritten_changes_iter, state_stream);

    // Filter out the keys that were deleted in the unwritten_changes cache.
    let filtered = merged.filter_map(|r| async move {
        r.map(|(k, v)| if v.is_some() { Some(k) } else { None })
            .transpose()
    });

    Box::pin(filtered)
}

#[allow(clippy::type_complexity)]
pub(crate) fn nonconsensus_prefix_raw_with_cache<'a>(
    sr: &'a impl StateRead,
    cache: &'a BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    prefix: &'a [u8],
) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>> {
    // Interleave the unwritten_changes cache with the snapshot.
    let state_stream = sr
        .nonconsensus_prefix_raw(prefix)
        .map(move |r| r.map(move |(k, v)| (k, Some(v))));

    // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
    // until we reach the keys that no longer match the prefix.
    let unwritten_changes_iter = cache
        .range(prefix.to_vec()..)
        .take_while(move |(k, _)| (**k).starts_with(prefix))
        .map(|(k, v)| (k.clone(), v.clone()));

    // Merge the cache iterator and state stream into a single stream.
    let merged = merge_cache(unwritten_changes_iter, state_stream);

    // Skip all the `None` values, as they were deleted.
    let merged =
        merged.filter_map(|r| async { r.map(|(k, v)| v.map(move |v| (k, v))).transpose() });

    Box::pin(merged)
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a S {
    type GetRawFut = S::GetRawFut;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'b>> {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'b>> {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw<'b>(
        &'b self,
        prefix: &'b [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'b>> {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        (**self).object_get(key)
    }
}

impl<'a, S: StateRead + Send + Sync> StateRead for &'a mut S {
    type GetRawFut = S::GetRawFut;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        (**self).get_raw(key)
    }

    fn prefix_raw<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'b>> {
        (**self).prefix_raw(prefix)
    }

    fn prefix_keys<'b>(
        &'b self,
        prefix: &'b str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'b>> {
        (**self).prefix_keys(prefix)
    }

    fn nonconsensus_prefix_raw<'b>(
        &'b self,
        prefix: &'b [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'b>> {
        (**self).nonconsensus_prefix_raw(prefix)
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        (**self).nonconsensus_get_raw(key)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        (**self).object_get(key)
    }
}
