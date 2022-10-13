use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use jmt::{
    storage::{LeafNode, Node, NodeBatch, NodeKey, TreeReader, TreeWriter},
    JellyfishMerkleTree, KeyHash,
};
use parking_lot::RwLock;
use penumbra_tct::Tree;
use rocksdb::{Options, DB};
use tracing::Span;

use crate::metrics;
use crate::snapshot::Snapshot;
use crate::State;
use ::metrics::gauge;

pub struct Storage {
    latest_snapshot: RwLock<Arc<Snapshot>>,
    db: &'static DB,
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

                    let db = Box::new(DB::open_cf(&opts, path, ["jmt", "nct", "indices"])?);
                    let static_db: &'static DB = Box::leak(db);
                    let jmt_version = latest_version(static_db).unwrap().unwrap();
                    let latest_snapshot = {
                        let snap = static_db.snapshot();
                        Snapshot::new(snap, jmt_version, static_db)
                    };

                    Ok(Self {
                        latest_snapshot: RwLock::new(Arc::new(latest_snapshot)),
                        db: static_db,
                    })
                })
            })
            .unwrap()
            .await
            .unwrap()
    }

    /// Returns the latest version (block height) of the tree recorded by the
    /// `Storage`, or `None` if the tree is empty.
    pub async fn latest_version(&self) -> Result<Option<jmt::Version>> {
        // TODO: do better
        Ok(latest_version(self.db).unwrap())
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub async fn state(&self) -> State {
        State::new(self.latest_snapshot.read().clone())
    }

    // TODO: this probably can't be 'static long-term
    pub async fn apply(&'static mut self, state: State, nct: &Tree) -> Result<()> {
        // 1. Write the NCT
        tracing::debug!("serializing NCT");
        let tct_data = bincode::serialize(nct)?;
        tracing::debug!(tct_bytes = tct_data.len(), "serialized NCT");
        gauge!(metrics::NCT_SIZE_BYTES, tct_data.len() as f64);

        let db = self.db;

        let span = Span::current();
        tokio::task::Builder::new()
            .name("put_nct")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let nct_cf = db.cf_handle("nct").expect("nct column family not found");
                    db.put_cf(nct_cf, "tct", &tct_data)
                })
            })
            .unwrap()
            .await??;

        // 2. Write the JMT to RocksDB
        // We use wrapping_add here so that we can write `new_version = 0` by
        // overflowing `PRE_GENESIS_VERSION`.
        let old_version = self.latest_version().await?.unwrap();
        let new_version = old_version.wrapping_add(1);
        tracing::trace!(old_version, new_version);
        let span = Span::current();
        tokio::task::Builder::new()
            .name("Storage::write_node_batch")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let snap = self.latest_snapshot.read().clone();
                    let jmt = JellyfishMerkleTree::new(snap.as_ref());
                    // Write the unwritten changes from the state to the JMT.
                    let (jmt_root_hash, batch) = jmt.put_value_set(
                        state
                            .unwritten_changes
                            .into_iter()
                            .map(|x| (KeyHash::from(x.0), x.1))
                            .collect(),
                        new_version,
                    )?;

                    // Apply the JMT changes to the DB.
                    self.write_node_batch(&batch.node_batch)?;
                    tracing::trace!(?jmt_root_hash, "wrote node batch to backing store");

                    // TODO: 3. write the index tables to RocksDB
                    // not yet implemented because the API for vectorized keys is not yet designed

                    // 4. update the snapshot
                    // Now that we've successfully written the new nodes, update the version.
                    let jmt_version = new_version;
                    let snapshot = db.snapshot();
                    // Obtain the write-lock for the latest snapshot, and replace it with the new snapshot.
                    let mut guard = self.latest_snapshot.write();
                    *guard = Arc::new(Snapshot::new(snapshot, jmt_version, db));
                    // Drop the write-lock (this will happen implicitly anyways, but it's good to be explicit).
                    drop(guard);
                    anyhow::Result::<()>::Ok(())
                })
            })
            .unwrap()
            .await
            .unwrap();

        Ok(())
    }
}

impl TreeWriter for Storage {
    /// Writes a node batch into storage.
    //TODO: Change JMT traits to accept owned NodeBatch
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let db = self.db;
        let node_batch = node_batch.clone();

        // The writes have to happen on a separate spawn_blocking task, but we
        // want tracing events to occur in the context of the current span, so
        // propagate it explicitly:
        let span = Span::current();

        for (node_key, node) in node_batch.clone() {
            let key_bytes = &node_key.encode()?;
            let value_bytes = &node.encode()?;
            tracing::trace!(?key_bytes, value_bytes = ?hex::encode(&value_bytes));

            let jmt_cf = db.cf_handle("jmt").expect("jmt column family not found");
            db.put_cf(jmt_cf, key_bytes, &value_bytes)?;
        }

        Ok(())
    }
}

// TODO: maybe these should live elsewhere?
fn get_rightmost_leaf(db: &DB) -> Result<Option<(NodeKey, LeafNode)>> {
    let span = Span::current();

    // TODO: should this be in a tokio task like before?
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
    Ok(get_rightmost_leaf(db)
        // TODO: do better
        .unwrap()
        .map(|(node_key, _)| node_key.version()))
}
