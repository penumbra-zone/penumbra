use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use jmt::{
    storage::{LeafNode, Node, NodeBatch, NodeKey, TreeWriter},
    JellyfishMerkleTree, KeyHash,
};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use tokio::sync::watch;
use tracing::Span;

use crate::snapshot::Snapshot;
use crate::State;

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
    latest_snapshot: RwLock<Snapshot>,
    db: Arc<DB>,
    state_tx: watch::Sender<StateNotification>,
}

/// A notification of a new state version.
pub struct StateNotification(Snapshot);

impl StateNotification {
    /// Obtain a snapshot of the new [`State`].
    pub fn into_state(&self) -> State {
        // We need this wrapper because the `State` itself isn't `Clone` (by design).
        State::new(self.0.clone())
    }
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
                        ["jmt", "nonconsensus", "jmt_keys"],
                    )?);

                    let jmt_version = latest_version(db.as_ref())?
                        // TODO: PRE_GENESIS_VERSION ?
                        .unwrap_or(u64::MAX);

                    let latest_snapshot = RwLock::new(Snapshot::new(db.clone(), jmt_version));

                    // We discard the receiver here, because we'll construct new ones in subscribe()
                    let (snapshot_tx, _) =
                        watch::channel(StateNotification(latest_snapshot.read().clone()));

                    Ok(Self(Arc::new(Inner {
                        latest_snapshot,
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
        self.0.latest_snapshot.read().version()
    }

    /// Returns a [`watch::Receiver`] that can be used to subscribe to new state versions.
    pub fn subscribe(&self) -> watch::Receiver<StateNotification> {
        // Calling subscribe() here to create a new receiver ensures
        // that all previous values are marked as seen, and the user
        // of the receiver will only be notified of *subsequent* values.
        self.0.state_tx.subscribe()
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub fn state(&self) -> State {
        State::new(self.0.latest_snapshot.read().clone())
    }

    /// Commits the provided [`State`] to persistent storage as the latest
    /// version of the chain state.
    pub async fn commit(&self, state: State) -> Result<crate::RootHash> {
        // We use wrapping_add here so that we can write `new_version = 0` by
        // overflowing `PRE_GENESIS_VERSION`.
        let old_version = self.latest_version();
        let new_version = old_version.wrapping_add(1);
        tracing::trace!(old_version, new_version);
        if old_version != state.version() {
            return Err(anyhow::anyhow!("version mismatch in commit: expected state forked from version {} but found state forked from version {}", old_version, state.version()));
        }

        let span = Span::current();
        let inner = self.0.clone();

        tokio::task::Builder::new()
            .name("Storage::write_node_batch")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let snap = inner.latest_snapshot.read().clone();
                    let jmt = JellyfishMerkleTree::new(&snap);

                    let unwritten_changes: Vec<_> = state
                        .unwritten_changes
                        .into_iter()
                        // Pre-calculate all KeyHashes for later storage in `jmt_keys`
                        .map(|x| (KeyHash::from(&x.0), x.0, x.1))
                        .collect();

                    // Write the JMT key lookups to RocksDB
                    let jmt_keys_cf = inner
                        .db
                        .cf_handle("jmt_keys")
                        .expect("jmt_keys column family not found");
                    for (keyhash, key_preimage, v) in unwritten_changes.iter() {
                        match v {
                            // Key still exists, so we need to store the key preimage
                            Some(_) => inner.db.put_cf(jmt_keys_cf, key_preimage, keyhash.0)?,
                            // Key was deleted, so delete the key preimage
                            None => {
                                inner.db.delete_cf(jmt_keys_cf, key_preimage)?;
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
                    for (k, v) in state.nonconsensus_changes.into_iter() {
                        let nonconsensus_cf = inner
                            .db
                            .cf_handle("nonconsensus")
                            .expect("nonconsensus column family not found");

                        match v {
                            Some(v) => inner.db.put_cf(nonconsensus_cf, k, &v)?,
                            None => {
                                inner.db.delete_cf(nonconsensus_cf, k)?;
                            }
                        };
                    }

                    // 4. update the snapshot

                    // Obtain the write-lock for the latest snapshot, and replace it with a new snapshot with the new version.
                    let mut guard = inner.latest_snapshot.write();
                    *guard = Snapshot::new(inner.db.clone(), new_version);
                    // Drop the write-lock (this will happen implicitly anyways, but it's good to be explicit).
                    drop(guard);

                    // .send fails if the channel is closed (i.e., if there are no receivers);
                    // in this case, we should ignore the error, we have no one to notify.
                    let _ = inner
                        .state_tx
                        .send(StateNotification(inner.latest_snapshot.read().clone()));

                    Ok(root_hash)
                })
            })?
            .await?
    }
}

impl TreeWriter for Inner {
    /// Writes a node batch into storage.
    //TODO: Change JMT traits to accept owned NodeBatch
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let node_batch = node_batch.clone();

        for (node_key, node) in node_batch {
            let key_bytes = &node_key.encode()?;
            let value_bytes = &node.encode()?;
            tracing::trace!(?key_bytes, value_bytes = ?hex::encode(&value_bytes));

            let jmt_cf = self
                .db
                .cf_handle("jmt")
                .expect("jmt column family not found");
            self.db.put_cf(jmt_cf, key_bytes, &value_bytes)?;
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
