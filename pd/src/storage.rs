use anyhow::Result;
use futures::future::BoxFuture;
use jmt::storage::{Node, NodeBatch, NodeKey, TreeReader, TreeWriter};
use rocksdb::DB;
use std::{path::PathBuf, sync::Arc};
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Storage(Arc<DB>);

impl Storage {
    pub async fn load(path: PathBuf) -> Result<Self> {
        tokio::task::spawn_blocking(|| Ok(Self(Arc::new(DB::open_default(path)?))))
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

        Box::pin(async {
            tokio::task::spawn_blocking(move || {
                for (node_key, node) in node_batch.clone() {
                    let key_bytes = &node_key.encode()?;
                    let value_bytes = &node.encode()?;
                    tracing::info!(?key_bytes, ?value_bytes);
                    db.put(key_bytes, value_bytes)?;
                }

                Ok(())
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

        Box::pin(async {
            tokio::task::spawn_blocking(move || {
                let value = match db.get_pinned(&node_key.encode()?) {
                    Ok(Some(value)) => {
                        let node = Node::decode(&value)?;
                        Some(node)
                    }
                    _ => None,
                };

                tracing::info!(?node_key, ?value);

                Ok(value)
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
