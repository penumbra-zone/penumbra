use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use futures::future::BoxFuture;
use jmt::storage::{Node, NodeBatch, NodeKey, TreeReader, TreeWriter};
use rocksdb::DB;
use tracing::{instrument, Span};

mod penumbra_store;
mod write_overlay_ext;

pub use penumbra_store::PenumbraStore;
pub use write_overlay_ext::WriteOverlayExt;

#[derive(Clone, Debug)]
pub struct Storage(Arc<DB>);

impl Storage {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let span = Span::current();
        tokio::task::spawn_blocking(move || {
            span.in_scope(|| {
                tracing::info!(?path, "opening rocksdb");
                Ok(Self(Arc::new(DB::open_default(path)?)))
            })
        })
        .await
        .unwrap()
    }
}

impl TreeWriter for Storage {
    /// Writes a node batch into storage.
    //TODO: Change JMT traits to remove/simplify lifetimes & accept owned NodeBatch
    #[instrument(skip(self, node_batch))]
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
            tokio::task::spawn_blocking(move || {
                span.in_scope(|| {
                    for (node_key, node) in node_batch.clone() {
                        let key_bytes = &node_key.encode()?;
                        let value_bytes = &node.encode()?;
                        tracing::debug!(?key_bytes, value_bytes = ?hex::encode(&value_bytes));
                        db.put(key_bytes, value_bytes)?;
                    }

                    Ok(())
                })
            })
            .await
            .unwrap()
        })
    }
}

/// A reader interface for rocksdb. NOTE: it is up to the caller to ensure consistency between the
/// rocksdb::DB handle and any write batches that may be applied through the writer interface.
impl TreeReader for Storage {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    #[instrument(skip(self))]
    fn get_node_option<'future, 'a: 'future, 'n: 'future>(
        &'a self,
        node_key: &'n NodeKey,
    ) -> BoxFuture<'future, Result<Option<Node>>> {
        let db = self.0.clone();
        let node_key = node_key.clone();

        let span = Span::current();

        Box::pin(async {
            tokio::task::spawn_blocking(move || {
                span.in_scope(|| {
                    let value = db
                        .get_pinned(&node_key.encode()?)?
                        .map(|db_slice| Node::decode(&db_slice))
                        .transpose()?;

                    tracing::debug!(?node_key, ?value);
                    Ok(value)
                })
            })
            .await
            .unwrap()
        })
    }

    fn get_rightmost_leaf<'future, 'a: 'future>(
        &'a self,
    ) -> BoxFuture<'future, Result<Option<(NodeKey, jmt::storage::LeafNode)>>> {
        todo!()
    }
}
