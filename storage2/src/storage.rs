use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use jmt::{
    storage::{LeafNode, Node, NodeBatch, NodeKey, TreeWriter},
    JellyfishMerkleTree, KeyHash,
};
use parking_lot::RwLock;
use rocksdb::{Options, DB};
use tracing::Span;

use crate::snapshot::Snapshot;
use crate::State;

/// A handle for a storage instance, backed by RocksDB.
///
/// The handle is cheaply clonable; all clones share the same backing data store.
#[derive(Clone)]
pub struct Storage(Arc<Inner>);

// A private inner element to prevent the `TreeWriter` implementation
// from leaking outside of this crate.
struct Inner {
    latest_snapshot: RwLock<Snapshot>,
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

                    let db = Arc::new(DB::open_cf(
                        &opts,
                        path,
                        ["jmt", "nonconsensus", "jmt_keys"],
                    )?);

                    let jmt_version = latest_version(db.as_ref())?
                        // TODO: PRE_GENESIS_VERSION ?
                        .unwrap_or(u64::MAX);

                    let latest_snapshot = RwLock::new(Snapshot::new(db.clone(), jmt_version));

                    Ok(Self(Arc::new(Inner {
                        latest_snapshot,
                        db,
                    })))
                })
            })?
            .await?
    }

    /// Returns the latest version (block height) of the tree recorded by the
    /// `Storage`, or `None` if the tree is empty.
    pub async fn latest_version(&self) -> Result<Option<jmt::Version>> {
        latest_version(&self.0.db)
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub fn state(&self) -> State {
        State::new(self.0.latest_snapshot.read().clone())
    }

    /// Commits the provided [`State`] to persistent storage as the latest
    /// version of the chain state.
    pub async fn commit(&self, state: State) -> Result<()> {
        let inner = self.0.clone();
        // 1. Write the NCT
        // TODO: move this higher up in the call stack, and use `put_nonconsensus` to store
        // the NCT.
        // tracing::debug!("serializing NCT");
        // let tct_data = bincode::serialize(nct)?;
        // tracing::debug!(tct_bytes = tct_data.len(), "serialized NCT");

        // let db = self.db;

        // let span = Span::current();
        // tokio::task::Builder::new()
        //     .name("put_nct")
        //     .spawn_blocking(move || {
        //         span.in_scope(|| {
        //             let nct_cf = db.cf_handle("nct").expect("nct column family not found");
        //             db.put_cf(nct_cf, "nct", &tct_data)
        //         })
        //     })
        //     .unwrap()
        //     .await??;

        // 2. Write the JMT and nonconsensus data to RocksDB
        // We use wrapping_add here so that we can write `new_version = 0` by
        // overflowing `PRE_GENESIS_VERSION`.
        let old_version = self.latest_version().await?.unwrap_or(u64::MAX);
        let new_version = old_version.wrapping_add(1);
        tracing::trace!(old_version, new_version);
        let span = Span::current();

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
                    let (jmt_root_hash, batch) = jmt.put_value_set(
                        unwritten_changes.into_iter().map(|x| (x.0, x.2)),
                        new_version,
                    )?;

                    // Apply the JMT changes to the DB.
                    inner.write_node_batch(&batch.node_batch)?;
                    tracing::trace!(?jmt_root_hash, "wrote node batch to backing store");

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
                    anyhow::Result::<()>::Ok(())
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
