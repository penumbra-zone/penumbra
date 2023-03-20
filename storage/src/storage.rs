use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use jmt::{
    storage::{LeafNode, Node, NodeBatch, NodeKey, TreeWriter},
    KeyHash, Sha256Jmt,
};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use sha2::Sha256;
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
    snapshots: RwLock<SnapshotCache>,
    db: Arc<DB>,
    state_tx: watch::Sender<Snapshot>,
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

                    let db = Arc::new(DB::open_cf(
                        &opts,
                        path,
                        // TODO(erwan): MERGEBLOCK - jmt still relevant now? maybe rename keys x keyhash
                        // add describition of each CFs
                        [
                            "jmt",
                            "jmt_values",
                            "jmt_keys",
                            "jmt_keys_by_keyhash",
                            "nonconsensus",
                        ],
                    )?);

                    // Note: for compatibility reasons with Tendermint, we set the "pre-genesis"
                    // jmt version to be u64::MAX, corresponding to -1 mod 2^64.
                    let jmt_version = latest_version(db.as_ref())?.unwrap_or(u64::MAX);

                    let latest_snapshot = Snapshot::new(db.clone(), jmt_version);

                    // We discard the receiver here, because we'll construct new ones in subscribe()
                    let (snapshot_tx, _) = watch::channel(latest_snapshot.clone());

                    let snapshots = RwLock::new(SnapshotCache::new(latest_snapshot, 10));

                    Ok(Self(Arc::new(Inner {
                        snapshots,
                        db,
                        state_tx: snapshot_tx,
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
        self.0.state_tx.subscribe()
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
                        .map(|x| (KeyHash::with::<Sha256>(&x.0), x.0, x.1))
                        .collect();

                    // Maintain a two-way index of the JMT keys and their hashes in RocksDB.
                    // The `jmt_keys` column family maps JMT `key`s to their `keyhash`.
                    // The `jmt_keys_by_keyhash` column family maps JMT `keyhash`es to their preimage.
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
                            // Key still exists, so we need to index its hash, and vice-versa.
                            Some(_) => {
                                inner.db.put_cf(jmt_keys_cf, key_preimage, keyhash.0)?;
                                inner
                                    .db
                                    .put_cf(jmt_keys_by_keyhash_cf, keyhash.0, key_preimage)?
                            }
                            // Key was deleted, so delete the key preimage, and its keyhash index.
                            None => {
                                inner.db.delete_cf(jmt_keys_cf, key_preimage)?;
                                inner.db.delete_cf(jmt_keys_by_keyhash_cf, keyhash.0)?;
                            }
                        };
                    }

                    // Write the unwritten changes from the state to the JMT.
                    let (root_hash, batch) = jmt.put_value_set(
                        unwritten_changes.into_iter().map(|x| (x.0, x.2)),
                        new_version,
                    )?;

                    // Apply the JMT changes to the DB.
                    inner.write_node_batch(&batch.node_batch)?;
                    tracing::trace!(?root_hash, "wrote node batch to backing store");

                    // Write the unwritten changes from the nonconsensus to RocksDB.
                    for (k, v) in cache.nonconsensus_changes.into_iter() {
                        let nonconsensus_cf = inner
                            .db
                            .cf_handle("nonconsensus")
                            .expect("nonconsensus column family not found");

                        match v {
                            Some(v) => {
                                tracing::trace!(key = ?EscapedByteSlice(&k), value = ?EscapedByteSlice(&v), "put nonconsensus key");
                                inner.db.put_cf(nonconsensus_cf, k, &v)?;
                            }
                            None => {
                                inner.db.delete_cf(nonconsensus_cf, k)?;
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
                    let _ = inner.state_tx.send(latest_snapshot);

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
            return Err(anyhow::anyhow!("version mismatch in commit: expected state forked from version {} but found state forked from version {}", old_version, snapshot.version()));
        }

        self.commit_inner(changes, new_version).await
    }

    /// Returns the internal handle to RocksDB, this is useful to test adjacent storage crates.
    #[cfg(test)]
    pub(crate) fn db(&self) -> Arc<DB> {
        self.0.db.clone()
    }
}

// TODO(erwan): move this somewhere? should this live in the jmt crate?
pub struct VersionedKey {
    version: jmt::Version,
    key: String,
}

impl TreeWriter for Inner {
    /// Writes a node batch into storage.
    //TODO: Change JMT traits to accept owned NodeBatch
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let node_batch = node_batch.clone();
        let jmt_cf = self
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");

        let jmt_values_cf = self
            .db
            .cf_handle("jmt_values")
            .expect("jmt_values column family not found");

        for (node_key, node) in node_batch.nodes() {
            let key_bytes = &node_key.encode()?;
            let node_bytes = &node.encode()?;
            tracing::trace!(?key_bytes, node_bytes = ?hex::encode(node_bytes));
            let key = String::from_utf8(key_bytes.to_vec())?;

            let versioned_key = VersionedKey {
                version: node_key.version(),
                key,
            };
            self.db.put_cf(jmt_cf, key_bytes, node_bytes)?;
            self.db.put_cf(jmt_values_cf, versioned_key, value)?;
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
        let node_key = NodeKey::decode(iter.key().unwrap())?;
        let node = Node::decode(iter.value().unwrap())?;

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
