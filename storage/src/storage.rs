use std::{path::PathBuf, sync::Arc};

use ::metrics::gauge;
use anyhow::Result;
use futures::future::BoxFuture;
use jmt::{
    storage::{Node, NodeBatch, NodeKey, TreeReader, TreeWriter},
    WriteOverlay,
};
use rocksdb::{Options, DB};
use tokio::sync::RwLock;
use tracing::Span;

use penumbra_tct as tct;

use crate::{metrics, State};

#[derive(Clone, Debug)]
pub struct Storage(Arc<DB>);

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

                    Ok(Self(Arc::new(DB::open_cf(&opts, path, ["jmt", "nct"])?)))
                })
            })
            .unwrap()
            .await
            .unwrap()
    }

    /// Returns the latest version (block height) of the tree recorded by the
    /// `Storage`, or `None` if the tree is empty.
    pub async fn latest_version(&self) -> Result<Option<jmt::Version>> {
        Ok(self
            .get_rightmost_leaf()
            .await?
            .map(|(node_key, _)| node_key.version()))
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub async fn state(&self) -> Result<State> {
        // If the tree is empty, use PRE_GENESIS_VERSION as the version,
        // so that the first commit will be at version 0.
        let version = self
            .latest_version()
            .await?
            .unwrap_or(WriteOverlay::<Storage>::PRE_GENESIS_VERSION);

        tracing::debug!("creating state for version {}", version);
        Ok(Arc::new(RwLock::new(WriteOverlay::new(
            self.clone(),
            version,
        ))))
    }

    /// Like [`Self::state`], but bundles in a [`tonic`] error conversion.
    ///
    /// This is useful for implementing gRPC services that query the storage:
    /// each gRPC request can create an ephemeral [`State`] pinning the current
    /// version at the time the request was received, and then query it using
    /// component `View`s to handle the request.
    pub async fn state_tonic(&self) -> std::result::Result<State, tonic::Status> {
        self.state()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }

    pub async fn put_nct(&self, tct: &tct::Tree) -> Result<()> {
        let db = self.0.clone();

        tracing::debug!("serializing TCT");
        let tct_data = bincode::serialize(tct)?;
        tracing::debug!(tct_bytes = tct_data.len(), "serialized TCT");
        gauge!(metrics::TCT_SIZE_BYTES, tct_data.len() as f64);

        let span = Span::current();
        tokio::task::Builder::new()
            .name("put_nct")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let nct_cf = db.cf_handle("nct").expect("nct column family not found");
                    db.put_cf(nct_cf, "tct", &tct_data)?;
                    Ok::<_, anyhow::Error>(())
                })
            })
            .unwrap()
            .await?
    }

    pub async fn get_nct(&self) -> Result<tct::Tree> {
        let db = self.0.clone();
        let span = Span::current();
        tokio::task::Builder::new()
            .name("get_nct")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let nct_cf = db.cf_handle("nct").expect("nct column family not found");
                    if let Some(tct_bytes) = db.get_cf(nct_cf, "tct")? {
                        Ok(bincode::deserialize(&tct_bytes)?)
                    } else {
                        Ok(tct::Tree::new())
                    }
                })
            })
            .unwrap()
            .await?
    }
}

impl TreeWriter for Storage {
    /// Writes a node batch into storage.
    //TODO: Change JMT traits to remove/simplify lifetimes & accept owned NodeBatch
    fn write_node_batch<'future, 'a: 'future, 'n: 'future>(
        &'a mut self,
        node_batch: &'n NodeBatch,
    ) -> BoxFuture<'future, Result<()>> {
        let db = self.0.clone();
        let node_batch = node_batch.clone();

        // The writes have to happen on a separate spawn_blocking task, but we
        // want tracing events to occur in the context of the current span, so
        // propagate it explicitly:
        let span = Span::current();

        Box::pin(async {
            tokio::task::Builder::new()
                .name("Storage::write_node_batch")
                .spawn_blocking(move || {
                    span.in_scope(|| {
                        for (node_key, node) in node_batch.clone() {
                            let key_bytes = &node_key.encode()?;
                            let value_bytes = &node.encode()?;
                            tracing::trace!(?key_bytes, value_bytes = ?hex::encode(&value_bytes));

                            let jmt_cf = db.cf_handle("jmt").expect("jmt column family not found");
                            db.put_cf(jmt_cf, key_bytes, &value_bytes)?;
                        }

                        Ok(())
                    })
                })
                .unwrap()
                .await
                .unwrap()
        })
    }
}

/// A reader interface for rocksdb. NOTE: it is up to the caller to ensure consistency between the
/// rocksdb::DB handle and any write batches that may be applied through the writer interface.
impl TreeReader for Storage {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option<'future, 'a: 'future, 'n: 'future>(
        &'a self,
        node_key: &'n NodeKey,
    ) -> BoxFuture<'future, Result<Option<Node>>> {
        let db = self.0.clone();
        let node_key = node_key.clone();

        let span = Span::current();

        Box::pin(async {
            tokio::task::Builder::new()
                .name("Storage::get_node_option")
                .spawn_blocking(move || {
                    span.in_scope(|| {
                        let jmt_cf = db.cf_handle("jmt").expect("jmt column family not found");
                        let value = db
                            .get_pinned_cf(jmt_cf, &node_key.encode()?)?
                            .map(|db_slice| Node::decode(&db_slice))
                            .transpose()?;

                        tracing::trace!(?node_key, ?value);
                        Ok(value)
                    })
                })
                .unwrap()
                .await
                .unwrap()
        })
    }

    fn get_rightmost_leaf<'future, 'a: 'future>(
        &'a self,
    ) -> BoxFuture<'future, Result<Option<(NodeKey, jmt::storage::LeafNode)>>> {
        let span = Span::current();
        let db = self.0.clone();

        Box::pin(async {
            tokio::task::Builder::new()
                .name("Storage::get_rightmost_leaf")
                .spawn_blocking(move || {
                    span.in_scope(|| {
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
                    })
                })
                .unwrap()
                .await
                .unwrap()
        })
    }
}
