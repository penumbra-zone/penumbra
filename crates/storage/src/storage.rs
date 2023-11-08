use std::{path::PathBuf, sync::Arc};
// use tokio_stream::wrappers::WatchStream;

use anyhow::{bail, Result};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use tokio::sync::watch;
use tracing::Span;

use crate::{
    cache::Cache,
    snapshot::Snapshot,
    store::{
        multistore::{self, MultistoreConfig},
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
    /// It is used by `Storage::release` to wait for the task to terminate.
    jh_dispatcher: Option<tokio::task::JoinHandle<()>>,
    db: Arc<DB>,
}

impl Storage {
    /// Initializes a new storage instance at the given path, along with a substore
    /// defined by each of the provided prefixes.
    pub async fn init(path: PathBuf, substore_prefixes: Vec<String>) -> Result<Self> {
        let span = Span::current();

        tokio::task::Builder::new()
            .name("open_rocksdb")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    tracing::info!(?path, "opening rocksdb");

                    let mut substore_configs = Vec::new();
                    tracing::info!("initializing global store config");
                    let main_store = Arc::new(SubstoreConfig::new(""));
                    for substore_prefix in substore_prefixes {
                        tracing::debug!(?substore_prefix, "reading substore config");
                        if substore_prefix.is_empty() {
                            bail!("the empty prefix is reserved")
                        }
                        substore_configs.push(Arc::new(SubstoreConfig::new(substore_prefix)));
                    }

                    let multistore_config = MultistoreConfig {
                        main_store: main_store.clone(),
                        substores: substore_configs.clone(),
                    };

                    // RocksDB setup: define options, collect all the columns, and open the database.
                    // Each substore defines a prefix and its own set of columns.
                    // See [`crate::store::SubstoreConfig`] for more details.
                    let mut opts = Options::default();
                    opts.create_if_missing(true);
                    opts.create_missing_column_families(true);

                    let mut substore_columns: Vec<&String> = substore_configs
                        .iter()
                        .flat_map(|config| config.columns())
                        .collect();
                    let mut columns: Vec<&String> = main_store.columns().collect();
                    columns.append(&mut substore_columns);

                    let db = Arc::new(DB::open_cf(&opts, path, columns)?);

                    // Initialize the substore cache with the latest version of each substore.
                    // Note: for compatibility reasons with Tendermint/CometBFT, we set the "pre-genesis"
                    // jmt version to be u64::MAX, corresponding to -1 mod 2^64.
                    let jmt_version = main_store.latest_version_from_db(&db)?.unwrap_or(u64::MAX);

                    let mut multistore_cache =
                        multistore::MultistoreCache::from_config(multistore_config.clone());

                    for substore_config in substore_configs {
                        let substore_version = substore_config
                            .latest_version_from_db(&db)?
                            .unwrap_or(u64::MAX);

                        multistore_cache.set_version(substore_config.clone(), substore_version);
                        tracing::debug!(
                            substore_prefix = ?substore_config.prefix,
                            ?substore_version,
                            "initializing substore"
                        );
                    }

                    multistore_cache.set_version(main_store, jmt_version);
                    tracing::debug!(?jmt_version, "initializing main store");

                    let latest_snapshot = Snapshot::new(db.clone(), jmt_version, multistore_cache);

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
                        multistore_config,
                        snapshots,
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
    /// version of the chain state.
    pub async fn commit(&self, delta: StateDelta<Snapshot>) -> Result<crate::RootHash> {
        // Extract the snapshot and the changes from the state delta
        let (snapshot, changes) = delta.flatten();

        // We use wrapping_add here so that we can write `new_version = 0` by
        // overflowing `PRE_GENESIS_VERSION`.
        let old_version = self.latest_version();
        let new_version = old_version.wrapping_add(1);
        tracing::debug!(old_version, new_version);
        if old_version != snapshot.version() {
            anyhow::bail!("version mismatch in commit: expected state forked from version {} but found state forked from version {}", old_version, snapshot.version());
        }

        self.commit_inner(snapshot, changes, new_version, true)
            .await
    }

    /// Commits the provided [`StateDelta`] to persistent storage as the latest
    /// version of the chain state. If `write_to_snapshot_cache` is `false`, the
    /// snapshot will not be written to the snapshot cache, and no subscribers
    /// will be notified.
    async fn commit_inner(
        &self,
        snapshot: Snapshot,
        cache: Cache,
        version: jmt::Version,
        write_to_snapshot_cache: bool,
    ) -> Result<crate::RootHash> {
        tracing::debug!(new_jmt_version = ?version, "committing state delta");
        let mut changes_by_substore = cache.shard_by_prefix(&self.0.multistore_config);
        let mut substore_roots = Vec::new();
        let mut multistore_versions =
            multistore::MultistoreCache::from_config(self.0.multistore_config.clone());

        let db = self.0.db.clone();
        let rocksdb_snapshot = snapshot.0.snapshot.clone();

        let mut new_versions = vec![];

        // We use a single write batch to commit all the substores at once. Each task will append
        // its own changes to the batch, and we will commit it at the end.
        let mut write_batch = rocksdb::WriteBatch::default();

        // Note(erwan): since we work over sharded keyspaces/disjoint column families,
        // it is tempting to consider a rewrite of this loop into a [`tokio::task::JoinSet`],
        // however there are some complications that should be on your radar:
        // * overhead: at the time of writing, there is a single digit number of substores,
        //   so it's implausible that the overhead of a joinset would be worth it.
        // * atomicity: unfortunately, `WriteBatch`es are not thread safe, this means that
        //   to spin-up N tasks, we would either need to:
        //      Option A: use a single batch, and synchronize access to it between tasks.
        //                if the number of substore contention grows with the number of substores,
        //                which is likely the case if you're considering a joinset.
        //      Option B: use N batches, and find a way to commit to them atomically.
        //                (better, but not supported) RocksDB does not allow merging batches
        //                together, and though [`rocksdb::OptimisticTransactionDB`] offers an
        //                ACID API, it is not compatible with the [`rocksdb::WriteBatch`] API.
        // A last option is to relax atomicity constraints, so that each commit task can produce
        // its own independent batch, and we can commit them all at once. This means that each batch
        // write is atomic, but the overall commit is not.
        for config in self.0.multistore_config.iter() {
            tracing::debug!(substore_prefix = ?config.prefix, "processing substore");
            // If the substore is empty, we need to fetch its initialized version from the cache.
            let old_substore_version = config
                .latest_version_from_snapshot(&db, &rocksdb_snapshot)?
                .unwrap_or_else(|| {
                    tracing::debug!("substore is empty, fetching initialized version from cache");
                    snapshot
                        .substore_version(config)
                        .expect("prefix should be initialized")
                });

            let Some(changeset) = changes_by_substore.remove(config) else {
                tracing::debug!(prefix = config.prefix, "no changes for substore, skipping");
                multistore_versions.set_version(config.clone(), old_substore_version);
                continue;
            };

            let version = old_substore_version.wrapping_add(1);
            new_versions.push(version);
            let substore_snapshot = SubstoreSnapshot {
                config: config.clone(),
                rocksdb_snapshot: rocksdb_snapshot.clone(),
                version,
                db: db.clone(),
            };

            let substore_storage = SubstoreStorage { substore_snapshot };

            // Commit the substore and collect the root hash
            let (root_hash, substore_batch) = substore_storage
                .commit(changeset, write_batch, version)
                .await?;
            write_batch = substore_batch;

            tracing::debug!(
                ?root_hash,
                prefix = config.prefix,
                ?version,
                "added substore to write batch"
            );
            substore_roots.push((config.clone(), root_hash, version));
        }

        /* commit roots to main store */
        let main_store_config = self.0.multistore_config.main_store.clone();
        let mut main_store_changes = changes_by_substore
            .remove(&main_store_config)
            .unwrap_or_else(|| {
                tracing::debug!("no changes for main store, creating empty changeset");
                Cache::default()
            });

        for (config, root_hash, _) in substore_roots.iter() {
            main_store_changes
                .unwritten_changes
                .insert(config.prefix.to_string(), Some(root_hash.0.to_vec()));
        }

        /* commit main substore */
        let main_store_snapshot = SubstoreSnapshot {
            config: main_store_config.clone(),
            rocksdb_snapshot: snapshot.0.snapshot.clone(),
            version,
            db: self.0.db.clone(),
        };

        let main_store_storage = SubstoreStorage {
            substore_snapshot: main_store_snapshot,
        };

        let (global_root_hash, write_batch) = main_store_storage
            .commit(main_store_changes, write_batch, version)
            .await?;
        tracing::debug!(
            ?global_root_hash,
            ?version,
            "added main store to write batch"
        );
        db.write(write_batch).expect("can write to db");

        /* update multistore versions */
        for (config, root_hash, new_version) in substore_roots {
            tracing::debug!(
                ?root_hash,
                prefix = ?config.prefix,
                ?new_version,
                "updating substore version"
            );
            multistore_versions.set_version(config, new_version);
        }

        tracing::debug!(?global_root_hash, ?version, "updating main store version");
        multistore_versions.set_version(main_store_config, version);

        /* hydrate the snapshot cache */
        if !write_to_snapshot_cache {
            tracing::debug!("skipping snapshot cache update");
            return Ok(global_root_hash);
        }

        tracing::debug!("updating snapshot cache");

        let latest_snapshot = Snapshot::new(db.clone(), version, multistore_versions);
        // Obtain a write lock to the snapshot cache, and push the latest snapshot
        // available. The lock guard is implicitly dropped immediately.
        self.0
            .snapshots
            .write()
            .try_push(latest_snapshot.clone())
            .expect("should process snapshots with consecutive jmt versions");

        tracing::debug!(?version, "dispatching snapshot");

        // Send fails if the channel is closed (i.e., if there are no receivers);
        // in this case, we should ignore the error, we have no one to notify.
        let _ = self.0.tx_dispatcher.send(latest_snapshot);

        Ok(global_root_hash)

        // The end
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

impl Inner {
    #[cfg(test)]
    pub async fn shutdown(&mut self) {
        if let Some(jh) = self.jh_dispatcher.take() {
            jh.abort();
            let _ = jh.await;
        }
    }
}
