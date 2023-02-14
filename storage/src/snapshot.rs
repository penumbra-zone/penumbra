use std::{any::Any, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use jmt::storage::{LeafNode, Node, NodeKey, TreeReader};
use tokio::sync::mpsc;
use tracing::Span;

use crate::state::StateRead;

mod rocks_wrapper;
use rocks_wrapper::RocksDbSnapshot;

/// A snapshot of the underlying storage at a specific state version, suitable
/// for read-only access by multiple threads, e.g., RPC calls.
///
/// Snapshots are cheap to create and clone.  Internally, they're implemented as
/// a wrapper around a [RocksDB
/// snapshot](https://github.com/facebook/rocksdb/wiki/Snapshot) with a pinned
/// JMT version number for the snapshot.
#[derive(Clone)]
pub struct Snapshot(Arc<Inner>);

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("version", &self.0.version)
            .finish_non_exhaustive()
    }
}

// We don't want to expose the `TreeReader` implementation outside of this crate.
#[derive(Debug)]
struct Inner {
    snapshot: RocksDbSnapshot,
    version: jmt::Version,
    // Used to retrieve column family handles.
    db: Arc<rocksdb::DB>,
}

impl Snapshot {
    pub(crate) fn new(db: Arc<rocksdb::DB>, version: jmt::Version) -> Self {
        Self(Arc::new(Inner {
            snapshot: RocksDbSnapshot::new(db.clone()),
            version,
            db,
        }))
    }

    pub fn version(&self) -> jmt::Version {
        self.0.version
    }

    /// Internal helper function used by `get_raw` and `prefix_raw`.
    ///
    /// Reads from the JMT will fail if the root is missing; this method
    /// special-cases the empty tree case so that reads on an empty tree just
    /// return None.
    fn get_jmt(&self, key: jmt::KeyHash) -> Result<Option<Vec<u8>>> {
        let tree = jmt::JellyfishMerkleTree::new(self);
        match tree.get(key, self.0.version) {
            Ok(Some(value)) => {
                tracing::trace!(version = ?self.0.version, ?key, value = ?hex::encode(&value), "read from tree");
                Ok(Some(value))
            }
            Ok(None) => {
                tracing::trace!(version = ?self.0.version, ?key, "key not found in tree");
                Ok(None)
            }
            // This allows for using the Overlay on an empty database without
            // errors We only skip the `MissingRootError` if the `version` is
            // `u64::MAX`, the pre-genesis version. Otherwise, a missing root
            // actually does indicate a problem.
            Err(e)
                if e.downcast_ref::<jmt::MissingRootError>().is_some()
                    && self.0.version == u64::MAX =>
            {
                tracing::trace!(version = ?self.0.version, "no data available at this version");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl StateRead for Snapshot {
    type GetRawFut = crate::future::SnapshotFuture;
    type PrefixRawStream =
        tokio_stream::wrappers::ReceiverStream<anyhow::Result<(String, Vec<u8>)>>;
    type PrefixKeysStream = tokio_stream::wrappers::ReceiverStream<anyhow::Result<String>>;
    type NonconsensusPrefixRawStream =
        tokio_stream::wrappers::ReceiverStream<anyhow::Result<(Vec<u8>, Vec<u8>)>>;

    /// Fetch a key from the JMT column family.
    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        let span = Span::current();
        let key_hash = jmt::KeyHash::from(key);
        let self2 = self.clone();
        crate::future::SnapshotFuture(
            tokio::task::Builder::new()
                .name("Snapshot::get_raw")
                .spawn_blocking(move || span.in_scope(|| self2.get_jmt(key_hash)))
                .expect("spawning threads is possible"),
        )
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        let span = Span::current();
        let inner = self.0.clone();
        let key: Vec<u8> = key.to_vec();
        crate::future::SnapshotFuture(
            tokio::task::Builder::new()
                .name("Snapshot::nonconsensus_get_raw")
                .spawn_blocking(move || {
                    span.in_scope(|| {
                        let nonconsensus_cf = inner
                            .db
                            .cf_handle("nonconsensus")
                            .expect("nonconsensus column family not found");
                        inner
                            .snapshot
                            .get_cf(nonconsensus_cf, key)
                            .map_err(Into::into)
                    })
                })
                .expect("spawning threads is possible"),
        )
    }

    fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream {
        let span = Span::current();
        let self2 = self.clone();

        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix.as_bytes()));
        let mode = rocksdb::IteratorMode::Start;

        let (tx, rx) = mpsc::channel(10);

        // Since the JMT keys are hashed, we can't use a prefix iterator directly.
        // We need to first prefix range the key preimages column family, then use the hashed matches to fetch the values
        // from the JMT column family.
        tokio::task::Builder::new()
            .name("Snapshot::prefix_raw")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let keys_cf = self2
                        .0
                        .db
                        .cf_handle("jmt_keys")
                        .expect("jmt_keys column family not found");
                    let iter = self2.0.snapshot.iterator_cf_opt(keys_cf, options, mode);
                    for i in iter {
                        // For each key that matches the prefix, fetch the value from the JMT column family.
                        let (key_preimage, _key_hash) = i?;
                        let k = std::str::from_utf8(key_preimage.as_ref())
                            .expect("saved jmt keys are utf-8 strings")
                            .to_string();
                        let v = self2
                            .get_jmt(k.as_bytes().into())?
                            .expect("keys in jmt_keys should have a corresponding value in jmt");
                        tx.blocking_send(Ok((k, v)))?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    // NOTE: this implementation is almost the same as the above, but without
    // fetching the values. not totally clear if this could be combined, or if that would
    // be better overall.
    fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream {
        let span = Span::current();
        let self2 = self.clone();

        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix.as_bytes()));
        let mode = rocksdb::IteratorMode::Start;

        let (tx, rx) = mpsc::channel(10);
        tokio::task::Builder::new()
            .name("Snapshot::prefix_keys")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let keys_cf = self2
                        .0
                        .db
                        .cf_handle("jmt_keys")
                        .expect("jmt_keys column family not found");
                    let iter = self2.0.snapshot.iterator_cf_opt(keys_cf, options, mode);
                    for i in iter {
                        // For each key that matches the prefix, fetch the value from the JMT column family.
                        let (key_preimage, _key_hash) = i?;
                        let k = std::str::from_utf8(key_preimage.as_ref())
                            .expect("saved jmt keys are utf-8 strings")
                            .to_string();
                        tx.blocking_send(Ok(k))?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    fn nonconsensus_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream {
        let span = Span::current();
        let self2 = self.clone();

        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix));
        let mode = rocksdb::IteratorMode::Start;

        let (tx, rx) = mpsc::channel(10);

        // Here we're operating on the nonconsensus data, which is a raw k/v store,
        // so we just iterate over the keys.
        tokio::task::Builder::new()
            .name("Snapshot::nonconsensus_prefix_raw")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let keys_cf = self2
                        .0
                        .db
                        .cf_handle("nonconsensus")
                        .expect("nonconsensus column family not found");
                    let iter = self2.0.snapshot.iterator_cf_opt(keys_cf, options, mode);
                    for i in iter {
                        let (key, value) = i?;
                        tx.blocking_send(Ok((key.into(), value.into())))?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, _key: &str) -> Option<T> {
        // No-op -- this will never be called internally, and `Snapshot` is not exposed in public API
        None
    }
}

/// A reader interface for rocksdb. NOTE: it is up to the caller to ensure consistency between the
/// rocksdb::DB handle and any write batches that may be applied through the writer interface.
impl TreeReader for Snapshot {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        let node_key = node_key;
        tracing::trace!(?node_key);

        let jmt_cf = self
            .0
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");
        let value = self
            .0
            .snapshot
            .get_cf(jmt_cf, &node_key.encode()?)?
            .map(|db_slice| Node::decode(&db_slice))
            .transpose()?;

        tracing::trace!(?node_key, ?value);
        Ok(value)
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>> {
        let jmt_cf = self
            .0
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");
        let mut iter = self.0.snapshot.raw_iterator_cf(jmt_cf);
        iter.seek_to_last();

        if iter.valid() {
            let node_key = NodeKey::decode(iter.key().unwrap())?;
            let node = Node::decode(iter.value().unwrap())?;

            if let Node::Leaf(leaf_node) = node {
                return Ok(Some((node_key, leaf_node)));
            }
        } else {
            // There are no keys in the database
        }

        Ok(None)
    }
}
