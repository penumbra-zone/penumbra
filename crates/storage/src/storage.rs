use borsh::BorshDeserialize;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
// use tokio_stream::wrappers::WatchStream;

use anyhow::{bail, Result};
use jmt::{
    storage::{LeafNode, NodeKey},
    KeyHash,
};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use tokio::sync::watch;
use tracing::Span;

use crate::{
    cache::Cache,
    snapshot::Snapshot,
    store::{
        multistore::MultistoreConfig,
        substore::{SubstoreConfig, SubstoreSnapshot, SubstoreStorage},
    },
};
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
    tx_dispatcher: watch::Sender<Snapshot>,
    tx_state: Arc<watch::Sender<Snapshot>>,
    snapshots: RwLock<SnapshotCache>,
    multistore_config: MultistoreConfig,
    #[allow(dead_code)]
    /// A handle to the dispatcher task.
    jh_dispatcher: Option<tokio::task::JoinHandle<()>>,
    db: Arc<DB>,
}

impl Storage {
    /// Initializes a new storage instance at the given path, along with a substore
    /// defined by each of the provided prefixes.
    /// TODO: potentially restore the `impl IntoIterator` if it makes sense later in the pr.
    pub async fn init(path: PathBuf, substore_prefixes: Vec<String>) -> Result<Self> {
        let span = Span::current();

        tokio::task::Builder::new()
            .name("open_rocksdb")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    tracing::info!(?path, "opening rocksdb");
                    let mut opts = Options::default();
                    opts.create_if_missing(true);
                    opts.create_missing_column_families(true);

                    let mut substore_configs = Vec::new();
                    tracing::info!("initializing global store config");
                    let main_store = SubstoreConfig::new("");
                    for substore_prefix in substore_prefixes {
                        tracing::info!(?substore_prefix, "initializing substore");
                        if substore_prefix.is_empty() {
                            bail!("the empty prefix is reserved")
                        }
                        substore_configs.push(Arc::new(SubstoreConfig::new(substore_prefix)));
                    }

                    let multistore_config = MultistoreConfig {
                        main_store: Arc::new(main_store),
                        substores: substore_configs.clone(),
                    };

                    let mut columns: Vec<&String> =
                        multistore_config.main_store.columns().collect();

                    let mut substore_columns: Vec<&String> = substore_configs
                        .iter()
                        .flat_map(|config| config.columns())
                        .collect();

                    columns.append(&mut substore_columns);

                    let db = Arc::new(DB::open_cf(&opts, path, columns)?);

                    // Note: for compatibility reasons with Tendermint/CometBFT, we set the "pre-genesis"
                    // jmt version to be u64::MAX, corresponding to -1 mod 2^64.
                    let jmt_version = latest_version(db.as_ref())?.unwrap_or(u64::MAX);

                    let latest_snapshot =
                        Snapshot::new(db.clone(), jmt_version, MultistoreConfig::default());

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
                        // We don't need to wrap the task in a `CancelOnDrop<T>` because
                        // the task will stop when the sender is dropped. However, certain
                        // test scenarios require us to wait that all resources are released.
                        jh_dispatcher: Some(jh_dispatcher),
                        tx_dispatcher,
                        tx_state,
                        snapshots,
                        multistore_config,
                        db,
                    })))
                })
            })?
            .await?
    }

    // TODO: hack, will consolidate later in the pr.
    pub async fn load(path: PathBuf) -> Result<Self> {
        Storage::init(path, vec![]).await
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

    /// Commits the provided [`StateDelta`] to persistent storage as the latest
    /// version of the chain state. If `write_to_snapshot_cache` is `false`, the
    /// snapshot will not be written to the snapshot cache, and no subscribers
    /// will be notified.
    async fn commit_inner(
        &self,
        snapshot: Snapshot,
        cache: Cache,
        new_version: jmt::Version,
        write_to_snapshot_cache: bool,
    ) -> Result<crate::RootHash> {
        let inner = self.0.clone();
        /* shard the set of changes */
        // TODO(erwan): pondering whether this should be encapsulated in a method over `Cache`. logically, i think this
        // makes sense but the current `Cache` impl is crisp and clean so i'm not sure.
        let mut changes_by_substore = BTreeMap::new();
        for (key, some_value) in cache.unwritten_changes.into_iter() {
            let (truncated_key, substore_config) = self.0.multistore_config.route_key_str(&key);
            changes_by_substore
                .entry(substore_config)
                .or_insert_with(|| Cache::default())
                .unwritten_changes
                .insert(truncated_key.to_string(), some_value);
        }

        for (key, some_value) in cache.nonverifiable_changes {
            let (truncated_key, substore_config) = self.0.multistore_config.route_key_bytes(&key);
            changes_by_substore
                .entry(substore_config)
                .or_insert_with(|| Cache::default())
                .nonverifiable_changes
                .insert(truncated_key.to_vec(), some_value);
        }

        /* commit the substores */
        let mut substore_roots = Vec::new();

        // TODO(erwan): the substores are disjoint key spaces and CFs
        // this is probably a good candidate for a joinset.
        for substore_config in &self.0.multistore_config.substores {
            let substore_snapshot = SubstoreSnapshot {
                config: substore_config.clone(),
                rocksdb_snapshot: snapshot.0.snapshot.clone(),
                version: new_version,
                db: self.0.db.clone(),
            };

            let substore_storage = SubstoreStorage {
                snapshot: substore_snapshot,
                config: substore_config.clone(),
                db: self.0.db.clone(),
            };

            // TODO(erwan): we want to atomically write the changes to the substore, and then write the root hash to the jmt
            // to do this we probably will have to pass a transaction object to the commit routine.
            let Some(changeset) = changes_by_substore.remove(substore_config) else {
                continue;
            };

            let root_hash = substore_storage.commit(changeset, new_version).await?;

            substore_roots.push((substore_config, root_hash));
        }

        /* commit roots to main store */
        let mut main_store_changes = changes_by_substore
            .remove(&self.0.multistore_config.main_store)
            .expect("always have main store changes"); // TODO(erwan): possibly relax this later in the pr, for testing.

        for (config, root_hash) in substore_roots {
            main_store_changes
                .unwritten_changes
                .insert(config.prefix.clone(), Some(root_hash.0.to_vec()));
        }

        /* commit main substore */
        let main_store_snapshot = SubstoreSnapshot {
            config: self.0.multistore_config.main_store.clone(),
            rocksdb_snapshot: snapshot.0.snapshot.clone(),
            version: new_version,
            db: self.0.db.clone(),
        };

        let main_store_storage = SubstoreStorage {
            config: main_store_snapshot.config.clone(),
            snapshot: main_store_snapshot,
            db: self.0.db.clone(),
        };

        let global_root_hash = main_store_storage
            .commit(main_store_changes, new_version)
            .await?;

        /* hydrate the snapshot cache */
        if !write_to_snapshot_cache {
            return Ok(global_root_hash);
        }

        let latest_snapshot = Snapshot::new(
            inner.db.clone(),
            new_version,
            inner.multistore_config.clone(),
        );
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

        Ok(global_root_hash)

        //end
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
        self.commit_inner(snapshot, changes, new_version, true)
            .await
    }

    #[cfg(feature = "migration")]
    /// Commits the provided [`StateDelta`] to persistent storage without increasing the version
    /// of the chain state.
    pub async fn commit_in_place(&self, delta: StateDelta<Snapshot>) -> Result<crate::RootHash> {
        let (snapshot, changes) = delta.flatten();
        let old_version = self.latest_version();
        self.commit_inner(snapshot, changes, old_version, false)
            .await
    }

    /// Returns the internal handle to RocksDB, this is useful to test adjacent storage crates.
    #[cfg(test)]
    pub(crate) fn db(&self) -> Arc<DB> {
        self.0.db.clone()
    }

    #[cfg(test)]
    /// Consumes the `Inner` storage and waits for all resources to be reclaimed.
    /// Panics if there are still outstanding references to the `Inner` storage.
    pub(crate) async fn release(mut self) {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            inner.shutdown().await;
            // `Inner` is dropped once the call completes.
        } else {
            panic!("Unable to get mutable reference to Inner");
        }
    }
}

// TODO: maybe these should live elsewhere?
fn get_rightmost_leaf(db: &DB) -> Result<Option<(NodeKey, LeafNode)>> {
    use crate::store::substore::DbNodeKey;
    use jmt::storage::Node;
    let cf_jmt = db
        .cf_handle("substore--jmt")
        .expect("jmt column family not found");
    let mut iter = db.raw_iterator_cf(cf_jmt);
    let mut ret = None;
    iter.seek_to_last();

    if iter.valid() {
        let node_key =
            DbNodeKey::decode(iter.key().expect("all DB entries should have a key"))?.into_inner();
        let node = Node::try_from_slice(iter.value().expect("all DB entries should have a value"))?;

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

impl Inner {
    #[cfg(test)]
    pub async fn shutdown(&mut self) {
        if let Some(jh) = self.jh_dispatcher.take() {
            jh.abort();
            let _ = jh.await;
        }
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
