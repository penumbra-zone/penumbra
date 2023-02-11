use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use futures::{FutureExt, Stream};
use tracing::Span;

mod cache;
mod read;
mod transaction;
mod write;

pub use read::StateRead;
pub use transaction::Transaction as StateTransaction;
pub use write::StateWrite;

use cache::Cache;

use crate::snapshot::Snapshot;

use self::read::{
    nonconsensus_prefix_raw_with_cache, prefix_keys_with_cache, prefix_raw_with_cache,
};

/// A lightweight snapshot of a particular version of the chain state.
///
/// Each [`State`] instance can also be used as a copy-on-write fork to build up
/// changes before committing them to persistent storage.  The
/// [`StateTransaction`] type collects a group of writes, which can then be
/// applied to the (in-memory) [`State`] fork.  Finally, the changes accumulated
/// in the [`State`] instance can be committed to the persistent
/// [`Storage`](crate::Storage).
///
/// The [`State`] type itself isn't `Clone`, to prevent confusion when a
/// [`State`] instance is used as a copy-on-write fork.  Either multiple
/// [`State`] instances should be forked from the underlying
/// [`Storage`](crate::Storage), if the states are meant to be independent, or
/// the [`State`] should be explicitly shared using an [`Arc`](std::sync::Arc).
pub struct State {
    pub(crate) snapshot: Snapshot,
    pub(crate) cache: Cache,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("snapshot", &self.snapshot)
            .field("dirty", &self.is_dirty())
            .finish_non_exhaustive()
    }
}

impl State {
    pub(crate) fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            cache: Default::default(),
        }
    }

    /// Begins a new batch of writes to be transactionally applied to this
    /// [`State`].
    ///
    /// The resulting [`StateTransaction`] captures a `&mut self` reference, so
    /// each [`State`] only allows one live transaction at a time.
    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
    }

    /// Returns the version this [`State`] snapshots.
    ///
    /// Note that there may be changes on top, if [`is_dirty`] returns `true`.
    pub fn version(&self) -> jmt::Version {
        self.snapshot.version()
    }

    /// Returns `true` if there are cached writes on top of the snapshot, and `false` otherwise.
    pub fn is_dirty(&self) -> bool {
        self.cache.is_dirty()
    }

    /// Gets a value by key alongside an ICS23 existence proof of that value.
    ///
    /// This method may only be used on a clean [`State`] fork, and will error
    /// if [`is_dirty`] returns `true`.
    ///
    /// Errors if the key is not present.
    /// TODO: change return type to `Option<Vec<u8>>` and an
    /// existence-or-nonexistence proof.
    pub async fn get_with_proof(&self, key: Vec<u8>) -> Result<(Vec<u8>, ics23::ExistenceProof)> {
        if self.is_dirty() {
            return Err(anyhow::anyhow!("requested get_with_proof on dirty State"));
        }
        let span = Span::current();
        let snapshot = self.snapshot.clone();

        tokio::task::Builder::new()
            .name("State::get_with_proof")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let tree = jmt::JellyfishMerkleTree::new(&snapshot);
                    let proof = tree.get_with_ics23_proof(key, snapshot.version())?;
                    Ok((proof.value.clone(), proof))
                })
            })?
            .await?
    }

    /// Returns the root hash of this `State`.
    ///
    /// If the `State` is empty, the all-zeros hash will be returned as a placeholder value.
    ///
    /// This method may only be used on a clean [`State`] fork, and will error
    /// if [`is_dirty`] returns `true`.
    pub async fn root_hash(&self) -> Result<crate::RootHash> {
        if self.is_dirty() {
            return Err(anyhow::anyhow!("requested root_hash on dirty State"));
        }
        let span = Span::current();
        let snapshot = self.snapshot.clone();

        tokio::task::Builder::new()
            .name("State::root_hash")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let tree = jmt::JellyfishMerkleTree::new(&snapshot);
                    let root = tree
                        .get_root_hash_option(snapshot.version())?
                        .unwrap_or(crate::RootHash([0; 32]));
                    Ok(root)
                })
            })?
            .await?
    }
}

#[async_trait]
impl StateRead for State {
    fn get_raw(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'static>> {
        // We want to return a 'static future, so we need to get all our references
        // to &self done upfront, before we bundle the results into a future.

        // If the key is available in the unwritten_changes cache, extract it now,
        // so we can move it into the future we'll return.
        let cached_value = self.cache.unwritten_changes.get(key).cloned();
        // Prepare a query to the state; this won't start executing until we poll it.
        let snapshot_value = self.snapshot.get_raw(key);

        async move {
            match cached_value {
                // If the key is available in the unwritten_changes cache, return it.
                Some(v) => Ok(v),
                // Otherwise, if the key is available in the JMT, return it.
                None => snapshot_value.await,
            }
        }
        .boxed()
    }

    fn nonconsensus_get_raw(
        &self,
        key: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'static>> {
        let cached_value = self.cache.nonconsensus_changes.get(key).cloned();
        let state_value = self.snapshot.nonconsensus_get_raw(key);
        async move {
            match cached_value {
                Some(v) => Ok(v),
                None => state_value.await,
            }
        }
        .boxed()
    }

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>> {
        prefix_raw_with_cache(&self.snapshot, &self.cache.unwritten_changes, prefix)
    }

    fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>> {
        prefix_keys_with_cache(&self.snapshot, &self.cache.unwritten_changes, prefix)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.cache
            .ephemeral_objects
            .get(key)
            .and_then(|maybe_object| maybe_object.as_ref())
            .and_then(|object| object.downcast_ref())
    }

    fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>> {
        nonconsensus_prefix_raw_with_cache(&self.snapshot, &self.cache.nonconsensus_changes, prefix)
    }
}

/// Extension trait providing `try_begin_transaction()` on `Arc<State>`.
pub trait ArcStateExt: Sized {
    /// Attempts to begin a transaction on this `Arc<State>`, returning `None` if the `Arc` is shared.
    fn try_begin_transaction(&'_ mut self) -> Option<StateTransaction<'_>>;
}

impl ArcStateExt for Arc<State> {
    fn try_begin_transaction(&'_ mut self) -> Option<StateTransaction<'_>> {
        Arc::get_mut(self).map(|state| state.begin_transaction())
    }
}
