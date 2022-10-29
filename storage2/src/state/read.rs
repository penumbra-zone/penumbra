use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug, pin::Pin};

use anyhow::Result;

use async_stream::stream;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_proto::{Message, Protobuf};

/// Read access to chain state.
#[async_trait]
// This needs to be a trait because we want to implement it over both `State` and `StateTransaction`,
// mainly to support RPC methods.
pub trait StateRead {
    /// Gets a value from the verifiable key-value store as raw bytes.
    ///
    /// Users should generally prefer to use [`get`](Self::get) or [`get_proto`](Self::get_proto).
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Gets a value from the verifiable key-value store as a domain type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a domain type `D`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a domain type `D`, or if an underlying storage error occurred.
    async fn get<D, P>(&self, key: &str) -> Result<Option<D>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        match self.get_proto(key).await {
            Ok(Some(p)) => match D::try_from(p) {
                Ok(d) => {
                    tracing::trace!(?key, value = ?d);
                    Ok(Some(d))
                }
                Err(e) => Err(e.into()),
            },
            Ok(None) => {
                tracing::trace!(?key, "no entry in tree");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Gets a value from the verifiable key-value store as a proto type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(v))` if the value is present and parseable as a proto type `P`;
    /// * `Ok(None)` if the value is missing;
    /// * `Err(_)` if the value is present but not parseable as a proto type `P`, or if an underlying storage error occurred.
    async fn get_proto<D, P>(&self, key: &str) -> Result<Option<P>>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let bytes = match self.get_raw(key).await? {
            None => return Ok(None),
            Some(bytes) => bytes,
        };

        Message::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!(e))
            .map(|v| Some(v))
    }

    /// Gets a byte value from the non-verifiable key-value store.
    ///
    /// This is intended for application-specific indexes of the verifiable
    /// consensus state, rather than for use as a primary data storage method.
    async fn get_nonconsensus(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Retrieve all values for keys matching a prefix from consensus-critical state, as domain types.
    async fn prefix<'a, D, P>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, D)>> + Send + 'a>>
    where
        D: Protobuf<P>,
        P: Message + Default + 'static,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        Box::pin(self.prefix_proto(prefix).await.map(|p| match p {
            Ok(p) => match D::try_from(p.1) {
                Ok(d) => Ok((p.0, d)),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e),
        }))
    }

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as proto types.
    async fn prefix_proto<'a, D, P>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, P)>> + Send + 'a>>
    where
        D: Protobuf<P>,
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        let o = self.prefix_raw(prefix).await.map(|r| {
            r.and_then(|(key, bytes)| {
                Ok((
                    key,
                    Message::decode(&*bytes).map_err(|e| anyhow::anyhow!(e))?,
                ))
            })
        });
        Box::pin(o)
    }

    /// Retrieve all values for keys matching a prefix from the verifiable key-value store, as raw bytes.
    ///
    /// Users should generally prefer to use [`prefix`](Self::prefix) or [`prefix_proto`](Self::prefix_proto).
    async fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
        // TODO: it might be possible to make this zero-allocation by representing the key as a `Box<&str>` but
        // the lifetimes weren't working out, so allocating a new `String` was easier for now.
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Box<[u8]>)>> + Sync + Send + 'a>>;
}

// Merge a RYW cache iterator with a backend storage stream to produce a new Stream,
// preferring results from the cache when keys are equal.
fn merge_cache<'a, K, V>(
    cache: impl Iterator<Item = (K, V)> + Send + Sync + Unpin + 'a,
    storage: impl Stream<Item = Result<(K, V)>> + Send + Sync + Unpin + 'a,
) -> impl Stream<Item = Result<(K, V)>> + Send + Sync + Unpin + 'a
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

pub(crate) async fn prefix_raw_with_cache<'a>(
    sr: &'a impl StateRead,
    cache: &'a BTreeMap<String, Option<Vec<u8>>>,
    prefix: &'a str,
) -> Pin<Box<dyn Stream<Item = Result<(String, Box<[u8]>)>> + Send + Sync + 'a>> {
    // Interleave the unwritten_changes cache with the snapshot.
    let state_stream = sr
        .prefix_raw(prefix)
        .await
        .map(move |r| r.map(move |(k, v)| (k, Some(v))));

    // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
    // until we reach the keys that no longer match the prefix.
    let unwritten_changes_iter = cache
        .range(prefix.to_string()..)
        .take_while(move |(k, _)| (**k).starts_with(prefix))
        .map(|(k, v)| {
            (
                k.clone(),
                v.as_ref().map(move |v| v.clone().into_boxed_slice()),
            )
        });

    // Maybe it would be possible to simplify this by using `async-stream` and implementing something similar to `itertools::merge_by`.
    let merged = merge_cache(unwritten_changes_iter, state_stream);

    // Skip all the `None` values, as they were deleted.
    let merged =
        merged.filter_map(|r| async { r.map(|(k, v)| v.map(move |v| (k, v))).transpose() });

    Box::pin(merged)
}
