use borsh::{BorshDeserialize, BorshSerialize};
use std::{path::PathBuf, sync::Arc};
// use tokio_stream::wrappers::WatchStream;

use anyhow::Result;
use jmt::{
    storage::{LeafNode, Node, NodeBatch, NodeKey, TreeWriter},
    KeyHash, Sha256Jmt,
};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use tokio::sync::watch;
use tracing::Span;

use crate::{cache::Cache, snapshot::Snapshot, EscapedByteSlice};
use crate::{snapshot_cache::SnapshotCache, StateDelta};

mod temp;
pub use temp::TempStorage;

/// A handle for a storage instance, backed by RocksDB.
///
/// The handle is cheaply clonable; all clones share the same backing data store.
#[derive(Clone)]
pub struct Storage(Arc<Inner>);

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage").finish_non_exhaustive()
    }
}

// A private inner element to prevent the `TreeWriter` implementation
// from leaking outside of this crate.
struct Inner {
    /// A handle to the dispatcher task.
    _jh_dispatcher: CancelOnDrop<()>,
    tx_dispatcher: watch::Sender<Snapshot>,
    tx_state: Arc<watch::Sender<Snapshot>>,
    snapshots: RwLock<SnapshotCache>,
    db: Arc<DB>,
}

impl Storage {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let span = Span::current();
        tokio::task::Builder::new()
            .name("open_rocksdb")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    tracing::info!(?path, "opening rocksdb");
                    let mut opts = Options::default();
                    opts.create_if_missing(true);
                    opts.create_missing_column_families(true);

                    // RocksDB columns:
                    let db = Arc::new(DB::open_cf(
                        &opts,
                        path,
                        [
                            // jmt: maps `storage::DbNodeKey` to `jmt::Node`, persists the internal structure of the JMT.
                            // Note: we need to use a newtype wrapper around `NodeKey` here, because
                            // we want a lexicographical ordering that maps to ascending jmt::Version.
                            "jmt",
                            // nonverifiable: maps arbitrary keys to arbitrary values, persists
                            // the nonverifiable state.
                            "nonverifiable",
                            // jmt_keys: index JMT keys (i.e. keyhash preimages).
                            "jmt_keys",
                            // jmt_keys_by_keyhash: index JMT keys by their hash.
                            "jmt_keys_by_keyhash",
                            // jmt_values: maps KeyHash || BE(version) to an `Option<Vec<u8>>`
                            "jmt_values",
                        ],
                    )?);

                    // Note: for compatibility reasons with Tendermint, we set the "pre-genesis"
                    // jmt version to be u64::MAX, corresponding to -1 mod 2^64.
                    let jmt_version = latest_version(db.as_ref())?.unwrap_or(u64::MAX);

                    let latest_snapshot = Snapshot::new(db.clone(), jmt_version);

                    // A concurrent-safe ring buffer of the latest 10 snapshots.
                    let snapshots = RwLock::new(SnapshotCache::new(latest_snapshot.clone(), 10));

                    // Setup a dispatcher task that acts as an intermediary between the storage
                    // and the rest of the system. Its purpose is to forward new snapshots to
                    // subscribers.
                    // If we were to send snapshots directly to subscribers, a slow subscriber could
                    // hold a lock on the watch channel for too long, and block the consensus-critical
                    // commit logic, which needs to acquire a write lock on the watch channel.
                    // dispatcher channel (internal):
                    // - `tx_dispatcher` is used by storage to signal that a new snapshot is available.
                    // - `rx_dispatcher` is used by the dispatcher to receive new snapshots.
                    // snapshot channel (external):
                    // - `tx_state` is used by the dispatcher to signal new snapshots to the rest of the system.
                    // - `rx_state` is used by various components to subscribe to new snapshots.
                    let (tx_state, _) = watch::channel(latest_snapshot.clone());
                    let (tx_dispatcher, mut rx_dispatcher) = watch::channel(latest_snapshot);

                    let tx_state = Arc::new(tx_state);
                    let tx_state2 = tx_state.clone();
                    let jh_dispatcher = tokio::spawn(async move {
                        tracing::info!("snapshot dispatcher task has started");
                        // If the sender is dropped, the task will terminate.
                        while rx_dispatcher.changed().await.is_ok() {
                            tracing::debug!("dispatcher has received a new snapshot");
                            let snapshot = rx_dispatcher.borrow_and_update().clone();
                            // [`watch::Sender<T>::send`] only returns an error if there are no
                            // receivers, so we can safely ignore the result here.
                            let _ = tx_state2.send(snapshot);
                        }
                        tracing::info!("dispatcher task has terminated")
                    });

                    Ok(Self(Arc::new(Inner {
                        _jh_dispatcher: CancelOnDrop::new(jh_dispatcher),
                        tx_dispatcher,
                        tx_state,
                        snapshots,
                        db,
                    })))
                })
            })?
            .await?
    }

    /// Returns the latest version (block height) of the tree recorded by the
    /// `Storage`.
    ///
    /// If the tree is empty and has not been initialized, returns `u64::MAX`.
    pub fn latest_version(&self) -> jmt::Version {
        self.latest_snapshot().version()
    }

    /// Returns a [`watch::Receiver`] that can be used to subscribe to new state versions.
    pub fn subscribe(&self) -> watch::Receiver<Snapshot> {
        // Calling subscribe() here to create a new receiver ensures
        // that all previous values are marked as seen, and the user
        // of the receiver will only be notified of *subsequent* values.
        self.0.tx_state.subscribe()
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub fn latest_snapshot(&self) -> Snapshot {
        self.0.snapshots.read().latest()
    }

    /// Fetches the [`State`] snapshot corresponding to the supplied `jmt::Version`
    /// from [`SnapshotCache`], or returns `None` if no match was found (cache-miss).
    pub fn snapshot(&self, version: jmt::Version) -> Option<Snapshot> {
        self.0.snapshots.read().get(version)
    }

    async fn commit_inner(
        &self,
        cache: Cache,
        new_version: jmt::Version,
    ) -> Result<crate::RootHash> {
        let span = Span::current();
        let inner = self.0.clone();

        tokio::task::Builder::new()
            .name("Storage::write_node_batch")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let snap = inner.snapshots.read().latest();
                    let jmt = Sha256Jmt::new(&*snap.0);

                    let unwritten_changes: Vec<_> = cache
                        .unwritten_changes
                        .into_iter()
                        // Pre-calculate all KeyHashes for later storage in `jmt_keys`
                        .map(|(key, some_value)| (KeyHash::with::<sha2::Sha256>(&key), key, some_value))
                        .collect();

                    // Maintain a two-way index of the JMT keys and their hashes in RocksDB.
                    // The `jmt_keys` column family maps JMT `key`s to their `keyhash`.
                    // The `jmt_keys_by_keyhash` column family maps JMT `keyhash`es to their preimage.
                    // Write the JMT key lookups to RocksDB
                    let jmt_keys_cf = inner
                        .db
                        .cf_handle("jmt_keys")
                        .expect("jmt_keys column family not found");

                    let jmt_keys_by_keyhash_cf = inner
                        .db
                        .cf_handle("jmt_keys_by_keyhash")
                        .expect("jmt_keys_by_keyhash family not found");

                    for (keyhash, key_preimage, v) in unwritten_changes.iter() {
                        match v {
                            // Key still exists so update the key preimage and keyhash index.
                            Some(_) => {
                                inner.db.put_cf(jmt_keys_cf, key_preimage, keyhash.0)?;
                                inner
                                    .db
                                    .put_cf(jmt_keys_by_keyhash_cf, keyhash.0, key_preimage)?
                            }
                            // Key was deleted, so delete the key preimage, and its keyhash index.
                            None => {
                                inner.db.delete_cf(jmt_keys_cf, key_preimage)?;
                                inner.db.delete_cf(jmt_keys_by_keyhash_cf, &keyhash.0)?;
                            }
                        };
                    }

                    // Write the unwritten changes from the state to the JMT.
                    let (root_hash, batch) = jmt.put_value_set(
                        unwritten_changes.into_iter().map(|(keyhash, _key, some_value)| (keyhash, some_value)),
                        new_version,
                    )?;

                    // Apply the JMT changes to the DB.
                    inner.write_node_batch(&batch.node_batch)?;
                    tracing::trace!(?root_hash, "wrote node batch to backing store");

                    // Write the unwritten changes from the nonverifiable to RocksDB.
                    for (k, v) in cache.nonverifiable_changes.into_iter() {
                        let nonverifiable_cf = inner
                            .db
                            .cf_handle("nonverifiable")
                            .expect("nonverifiable column family not found");

                        match v {
                            Some(v) => {
                                tracing::trace!(key = ?EscapedByteSlice(&k), value = ?EscapedByteSlice(&v), "put nonverifiable key");
                                inner.db.put_cf(nonverifiable_cf, k, &v)?;
                            }
                            None => {
                                inner.db.delete_cf(nonverifiable_cf, k)?;
                            }
                        };
                    }

                    let latest_snapshot = Snapshot::new(inner.db.clone(), new_version);
                    // Obtain a write lock to the snapshot cache, and push the latest snapshot
                    // available. The lock guard is implicitly dropped immediately.
                    inner
                        .snapshots
                        .write()
                        .try_push(latest_snapshot.clone())
                        .expect("should process snapshots with consecutive jmt versions");

                    // Send fails if the channel is closed (i.e., if there are no receivers);
                    // in this case, we should ignore the error, we have no one to notify.
                    let _ = inner.tx_dispatcher.send(latest_snapshot);

                    Ok(root_hash)
                })
            })?
            .await?
    }

    /// Commits the provided [`StateDelta`] to persistent storage as the latest
    /// version of the chain state.
    pub async fn commit(&self, delta: StateDelta<Snapshot>) -> Result<crate::RootHash> {
        // Extract the snapshot and the changes from the state delta
        let (snapshot, changes) = delta.flatten();

        // We use wrapping_add here so that we can write `new_version = 0` by
        // overflowing `PRE_GENESIS_VERSION`.
        let old_version = self.latest_version();
        let new_version = old_version.wrapping_add(1);
        tracing::trace!(old_version, new_version);
        if old_version != snapshot.version() {
            anyhow::bail!("version mismatch in commit: expected state forked from version {} but found state forked from version {}", old_version, snapshot.version());
        }

        self.commit_inner(changes, new_version).await
    }

    /// Returns the internal handle to RocksDB, this is useful to test adjacent storage crates.
    #[cfg(test)]
    pub(crate) fn db(&self) -> Arc<DB> {
        self.0.db.clone()
    }
}

impl TreeWriter for Inner {
    /// Writes a [`NodeBatch`] into storage which includes the JMT
    /// nodes (`DbNodeKey` -> `Node`) and the JMT values,
    /// (`VersionedKeyHash` -> `Option<Vec<u8>>`).
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let node_batch = node_batch.clone();
        let jmt_cf = self
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");

        for (node_key, node) in node_batch.nodes() {
            let db_node_key = DbNodeKey::from(node_key.clone());
            let db_node_key_bytes = db_node_key.encode()?;
            let value_bytes = &node.try_to_vec()?;
            tracing::trace!(?db_node_key_bytes, value_bytes = ?hex::encode(value_bytes));
            self.db.put_cf(jmt_cf, db_node_key_bytes, value_bytes)?;
        }
        let jmt_values_cf = self
            .db
            .cf_handle("jmt_values")
            .expect("jmt_values column family not found");

        for ((version, key_hash), some_value) in node_batch.values() {
            let versioned_key = VersionedKeyHash::new(*version, *key_hash);
            let key_bytes = &versioned_key.encode();
            let value_bytes = &some_value.try_to_vec()?;
            tracing::trace!(?key_bytes, value_bytes = ?hex::encode(value_bytes));

            self.db.put_cf(jmt_values_cf, key_bytes, value_bytes)?;
        }

        Ok(())
    }
}

// TODO: maybe these should live elsewhere?
fn get_rightmost_leaf(db: &DB) -> Result<Option<(NodeKey, LeafNode)>> {
    let jmt_cf = db.cf_handle("jmt").expect("jmt column family not found");
    let mut iter = db.raw_iterator_cf(jmt_cf);
    let mut ret = None;
    iter.seek_to_last();

    if iter.valid() {
        let node_key = DbNodeKey::decode(iter.key().unwrap())?.into_inner();
        let node = Node::try_from_slice(iter.value().unwrap())?;

        if let Node::Leaf(leaf_node) = node {
            ret = Some((node_key, leaf_node));
        }
    } else {
        // There are no keys in the database
    }

    Ok(ret)
}

pub fn latest_version(db: &DB) -> Result<Option<jmt::Version>> {
    Ok(get_rightmost_leaf(db)?.map(|(node_key, _)| node_key.version()))
}

pub struct CancelOnDrop<T>(tokio::task::JoinHandle<T>);

impl CancelOnDrop<()> {
    pub fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self(handle)
    }
}

impl<T> Drop for CancelOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Represent a JMT key hash at a specific `jmt::Version`
/// This is used to index the JMT values in RocksDB.
#[derive(Clone, Debug)]
pub struct VersionedKeyHash {
    pub key_hash: KeyHash,
    pub version: jmt::Version,
}

impl VersionedKeyHash {
    pub fn new(version: jmt::Version, key_hash: KeyHash) -> Self {
        Self { version, key_hash }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = self.key_hash.0.to_vec();
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf
    }

    pub fn _decode(buf: Vec<u8>) -> Result<Self> {
        if buf.len() != 40 {
            Err(anyhow::anyhow!(
                "could not decode buffer into VersionedKey (invalid size)"
            ))
        } else {
            let raw_key_hash: [u8; 32] = buf[0..32]
                .try_into()
                .expect("buffer is at least 40 bytes wide");
            let key_hash = KeyHash(raw_key_hash);

            let raw_version: [u8; 8] = buf[32..40]
                .try_into()
                .expect("buffer is at least 40 bytes wide");
            let version: u64 = u64::from_be_bytes(raw_version);

            Ok(VersionedKeyHash { version, key_hash })
        }
    }
}

/// An ordered node key is a node key that is encoded in a way that
/// preserves the order of the node keys in the database.
pub struct DbNodeKey(NodeKey);

impl DbNodeKey {
    pub fn from(node_key: NodeKey) -> Self {
        DbNodeKey(node_key)
    }

    pub fn into_inner(self) -> NodeKey {
        self.0
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.version().to_be_bytes()); // encode version as big-endian
        let rest = borsh::BorshSerialize::try_to_vec(&self.0)?;
        bytes.extend_from_slice(&rest);
        Ok(bytes)
    }

    pub fn decode(bytes: impl AsRef<[u8]>) -> Result<Self> {
        if bytes.as_ref().len() < 8 {
            anyhow::bail!("byte slice is too short")
        }
        // Ignore the bytes that encode the version
        let node_key_slice = bytes.as_ref()[8..].to_vec();
        let node_key = borsh::BorshDeserialize::try_from_slice(&node_key_slice)?;
        Ok(DbNodeKey(node_key))
    }
}
