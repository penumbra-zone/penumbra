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
    dispatcher_tx: watch::Sender<(Snapshot, (jmt::Version, Arc<Cache>))>,
    snapshot_rx: watch::Receiver<Snapshot>,
    changes_rx: watch::Receiver<(jmt::Version, Arc<Cache>)>,
    snapshots: RwLock<SnapshotCache>,
    multistore_config: MultistoreConfig,
    #[allow(dead_code)]
    /// A handle to the dispatcher task.
    /// This is used by `Storage::release` to wait for the task to terminate.
    jh_dispatcher: Option<tokio::task::JoinHandle<()>>,
    db: Arc<DB>,
}

impl Storage {
    /// Loads a storage instance from the given path, initializing it if necessary.
    pub async fn load(path: PathBuf, default_prefixes: Vec<String>) -> Result<Self> {
        let span = Span::current();
        let db_path = path.clone();
        // initializing main storage instance.
        let prefixes = tokio::task::Builder::new()
            .name("config_rocksdb")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let mut opts = Options::default();
                    opts.create_if_missing(true);
                    opts.create_missing_column_families(true);
                    tracing::info!(?path, "opening rocksdb config column");

                    // Hack(erwan): RocksDB requires us to specify all the column families
                    // that we want to use upfront. This is problematic when we are initializing
                    // a new database, because the call to `DBCommon<T>::list_cf` will fail
                    // if the database manifest is not found. To work around this, we ignore
                    // the error and assume that the database is empty.
                    // Tracked in: https://github.com/rust-rocksdb/rust-rocksdb/issues/608
                    let mut columns = DB::list_cf(&opts, path.clone()).unwrap_or_default();
                    if columns.is_empty() {
                        columns.push("config".to_string());
                    }

                    let db = DB::open_cf(&opts, path, columns).expect("can open database");
                    let cf_config = db
                        .cf_handle("config")
                        .expect("config column family is created if missing");
                    let config_iter = db.iterator_cf(cf_config, rocksdb::IteratorMode::Start);
                    let mut prefixes = Vec::new();
                    tracing::info!("reading prefixes from config column family");
                    for i in config_iter {
                        let (key, _) = i.expect("can read from iterator");
                        prefixes.push(String::from_utf8(key.to_vec()).expect("prefix is utf8"));
                    }

                    for prefix in default_prefixes {
                        if !prefixes.contains(&prefix) {
                            db.put_cf(cf_config, prefix.as_bytes(), b"")
                                .expect("can write to db");
                            prefixes.push(prefix);
                        }
                    }

                    std::mem::drop(db);
                    prefixes
                })
            })?
            .await?;

        Storage::init(db_path, prefixes).await
    }

    /// Initializes a new storage instance at the given path. Takes a list of default prefixes
    /// to initialize the storage configuration with.
    /// Here is a high-level overview of the initialization process:
    /// 1. Create a new RocksDB instance at the given path.
    /// 2. Read the prefix list and create a [`SubstoreConfig`] for each prefix.
    /// 3. Create a new [`MultistoreConfig`] from supplied prefixes.
    /// 4. Initialize the substore cache with the latest version of each substore.
    /// 5. Spawn a dispatcher task that forwards new snapshots to subscribers.
    pub async fn init(path: PathBuf, prefixes: Vec<String>) -> Result<Self> {
        let span = Span::current();

        tokio::task::Builder::new()
            .name("open_rocksdb")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let mut substore_configs = Vec::new();
                    tracing::info!("initializing global store config");
                    let main_store = Arc::new(SubstoreConfig::new(""));
                    for substore_prefix in prefixes {
                        tracing::info!(prefix = ?substore_prefix, "creating substore config for prefix");
                        if substore_prefix.is_empty() {
                            bail!("the empty prefix is reserved")
                        }
                        substore_configs.push(Arc::new(SubstoreConfig::new(substore_prefix)));
                    }

                    let multistore_config = MultistoreConfig {
                        main_store: main_store.clone(),
                        substores: substore_configs.clone(),
                    };

                    let mut substore_columns: Vec<&String> = substore_configs
                        .iter()
                        .flat_map(|config| config.columns())
                        .collect();
                    let mut columns: Vec<&String> = main_store.columns().collect();
                    columns.append(&mut substore_columns);

                    tracing::info!(?path, "opening rocksdb");
                    let cf_config_string = "config".to_string();
                    // RocksDB setup: define options, collect all the columns, and open the database.
                    // Each substore defines a prefix and its own set of columns.
                    // See [`crate::store::SubstoreConfig`] for more details.
                    let mut opts = Options::default();
                    opts.create_if_missing(true);
                    opts.create_missing_column_families(true);
                    columns.push(&cf_config_string);

                    let db = DB::open_cf(&opts, path, columns)?;
                    let shared_db = Arc::new(db);

                    // Initialize the substore cache with the latest version of each substore.
                    // Note: for compatibility reasons with Tendermint/CometBFT, we set the "pre-genesis"
                    // jmt version to be u64::MAX, corresponding to -1 mod 2^64.
                    let jmt_version = main_store
                        .latest_version_from_db(&shared_db)?
                        .unwrap_or(u64::MAX);

                    let mut multistore_cache =
                        multistore::MultistoreCache::from_config(multistore_config.clone());

                    for substore_config in substore_configs {
                        let substore_version = substore_config
                            .latest_version_from_db(&shared_db)?
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

                    let latest_snapshot =
                        Snapshot::new(shared_db.clone(), jmt_version, multistore_cache);

                    // A concurrent-safe ring buffer of the latest 10 snapshots.
                    let snapshots = RwLock::new(SnapshotCache::new(latest_snapshot.clone(), 10));

                    // Setup a dispatcher task that acts as an intermediary between the storage
                    // and the rest of the system. Its purpose is to forward new snapshots to
                    // subscribers.
                    //
                    // If we were to send snapshots directly to subscribers, a slow subscriber could
                    // hold a lock on the watch channel for too long, and block the consensus-critical
                    // commit logic, which needs to acquire a write lock on the watch channel.
                    //
                    // Instead, we "proxy" through a dispatcher task that copies values from one 
                    // channel to the other, ensuring that if an API consumer misuses the watch 
                    // channels, it will only affect other subscribers, not the commit logic.

                    let (snapshot_tx, snapshot_rx) = watch::channel(latest_snapshot.clone());
                    // Note: this will never be seen by consumers, since we mark the current value as seen
                    // before returning the receiver.
                    let dummy_cache = (u64::MAX, Arc::new(Cache::default()));
                    let (changes_tx, changes_rx) = watch::channel(dummy_cache.clone());
                    let (tx_dispatcher, mut rx_dispatcher) = watch::channel((latest_snapshot, dummy_cache));

                    let jh_dispatcher = tokio::spawn(async move {
                        tracing::info!("snapshot dispatcher task has started");
                        // If the sender is dropped, the task will terminate.
                        while rx_dispatcher.changed().await.is_ok() {
                            tracing::debug!("dispatcher has received a new snapshot");
                            let (snapshot, changes) = rx_dispatcher.borrow_and_update().clone();
                            // [`watch::Sender<T>::send`] only returns an error if there are no
                            // receivers, so we can safely ignore the result here.
                            let _ = snapshot_tx.send(snapshot);
                            let _ = changes_tx.send(changes);
                        }
                        tracing::info!("dispatcher task has terminated")
                    });

                    Ok(Self(Arc::new(Inner {
                        // We don't need to wrap the task in a `CancelOnDrop<T>` because
                        // the task will stop when the sender is dropped. However, certain
                        // test scenarios require us to wait that all resources are released.
                        jh_dispatcher: Some(jh_dispatcher),
                        dispatcher_tx: tx_dispatcher,
                        snapshot_rx,
                        changes_rx,
                        multistore_config,
                        snapshots,
                        db: shared_db,
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
        let mut rx = self.0.snapshot_rx.clone();
        // Mark the current value as seen, so that the user of the receiver
        // will only be notified of *subsequent* values.
        rx.borrow_and_update();
        rx
    }

    /// Returns a [`watch::Receiver`] that can be used to subscribe to state changes.
    pub fn subscribe_changes(&self) -> watch::Receiver<(jmt::Version, Arc<Cache>)> {
        let mut rx = self.0.changes_rx.clone();
        // Mark the current value as seen, so that the user of the receiver
        // will only be notified of *subsequent* values.
        rx.borrow_and_update();
        rx
    }

    /// Returns a new [`Snapshot`] on top of the latest version of the tree.
    pub fn latest_snapshot(&self) -> Snapshot {
        self.0.snapshots.read().latest()
    }

    /// Fetches the [`Snapshot`] corresponding to the supplied `jmt::Version` from
    /// the [`SnapshotCache`]. Returns `None` if no match was found.
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

        self.commit_inner(snapshot, changes, new_version, false)
            .await
    }

    /// Commits the supplied [`Cache`] to persistent storage.
    ///
    /// # Migrations
    /// In the case of chain state migrations we need to commit the new state
    /// without incrementing the version. If `perform_migration` is `true` the
    /// snapshot will _not_ be written to the snapshot cache, and no subscribers
    /// will be notified. Substore versions will not be updated.
    async fn commit_inner(
        &self,
        snapshot: Snapshot,
        cache: Cache,
        version: jmt::Version,
        perform_migration: bool,
    ) -> Result<crate::RootHash> {
        tracing::debug!(new_jmt_version = ?version, "committing state delta");
        // Save a copy of the changes to send to subscribers later.
        let changes = Arc::new(cache.clone_changes());

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

        // Note(erwan): Here, we iterate over each substore, and spawn a task to
        //  commit it. Since we know that the substore keyspace is disjoint, we
        //  could consider rewriting this loop into a [`tokio::task::JoinSet`],
        //  however consider that `rocksdb::WriteBatch` is _not_ thread-safe.
        //
        //  This means that to spin-up N tasks, we would need to use a
        //  single batch wrapped in a mutex, or use N batches, and find
        //  a way to commit to them atomically. Since that is not supported
        //  by RocksDB, we would have to iterate over each entry in each
        //  batch, and merge them together.
        //
        //  Another option is to trade atomicity for parallelism by producing
        //  N batches, and committing them in distinct atomic writes. This is
        //  dangerous because it could leave the node in an inconsistent state.
        //
        //  Instead of doing that, we lean on the fact that the number of substores
        //  is small, and that the synchronization overhead of a joinset would exceed
        //  its benefits.
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

            let version = if perform_migration {
                old_substore_version
            } else {
                old_substore_version.wrapping_add(1)
            };
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
        if perform_migration {
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
        let _ = self
            .0
            .dispatcher_tx
            .send((latest_snapshot, (version, changes)));

        Ok(global_root_hash)
    }

    #[cfg(feature = "migration")]
    /// Commits the provided [`StateDelta`] to persistent storage without increasing the version
    /// of the chain state.
    pub async fn commit_in_place(&self, delta: StateDelta<Snapshot>) -> Result<crate::RootHash> {
        let (snapshot, changes) = delta.flatten();
        let old_version = self.latest_version();
        self.commit_inner(snapshot, changes, old_version, true)
            .await
    }

    /// Returns the internal handle to RocksDB, this is useful to test adjacent storage crates.
    #[cfg(test)]
    pub(crate) fn db(&self) -> Arc<DB> {
        self.0.db.clone()
    }

    /// Shuts down the database and the dispatcher task, and waits for all resources to be reclaimed.
    /// Panics if there are still outstanding references to the `Inner` storage.
    pub async fn release(mut self) {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            inner.shutdown().await;
            inner.snapshots.write().clear();
            // `Inner` is dropped once the call completes.
        } else {
            panic!("Unable to get mutable reference to Inner");
        }
    }
}

impl Inner {
    pub(crate) async fn shutdown(&mut self) {
        if let Some(jh) = self.jh_dispatcher.take() {
            jh.abort();
            let _ = jh.await;
        }
    }
}
