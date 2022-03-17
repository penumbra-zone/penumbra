use anyhow::Result;
use futures::future::BoxFuture;
use jmt::storage::{LeafNode, Node, NodeBatch, NodeKey, TreeReader, TreeWriter};
use sqlx::{query, Postgres};

use crate::state;

/// Wrapper struct used to implement [`jmt::TreeWriterAsync`] for a Postgres
/// transaction, without violating the orphan rules.
pub struct DbTx<'conn, 'tx>(pub &'tx mut sqlx::Transaction<'conn, Postgres>);

impl<'conn, 'tx> TreeWriter for DbTx<'conn, 'tx> {
    /// Writes a node batch into storage.
    fn write_node_batch<'future, 'a: 'future, 'n: 'future>(
        &'a mut self,
        node_batch: &'n NodeBatch,
    ) -> BoxFuture<'future, Result<()>> {
        Box::pin(async move {
            for (node_key, node) in node_batch.clone() {
                let key_bytes = &node_key.encode()?;
                let value_bytes = &node.encode()?;

                query!(
                    r#"
                    INSERT INTO jmt (key, value) VALUES ($1, $2)
                    "#,
                    &key_bytes,
                    &value_bytes
                )
                .execute(&mut *self.0)
                .await?;
            }

            Ok(())
        })
    }
}

impl TreeReader for state::Reader {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option<'future, 'a: 'future, 'n: 'future>(
        &'a self,
        node_key: &'n NodeKey,
    ) -> BoxFuture<'future, Result<Option<Node>>> {
        Box::pin(async {
            let mut conn = self.pool.acquire().await?;

            let value = query!(
                r#"SELECT value FROM jmt WHERE key = $1 LIMIT 1"#,
                &node_key.encode()?
            )
            .fetch_optional(&mut conn)
            .await?;

            let value = match value {
                Some(row) => Some(Node::decode(&row.value)?),
                _ => None,
            };

            Ok(value)
        })
    }

    /// Gets the rightmost leaf. Note that this assumes we are in the process of restoring the tree
    /// and all nodes are at the same version.
    #[allow(clippy::type_complexity)]
    fn get_rightmost_leaf<'future, 'a: 'future>(
        &'a self,
    ) -> BoxFuture<'future, Result<Option<(NodeKey, LeafNode)>>> {
        Box::pin(async {
            let mut conn = self.pool.acquire().await?;

            let value = query!(r#"SELECT key, value FROM jmt ORDER BY key DESC LIMIT 1"#)
                .fetch_optional(&mut conn)
                .await?;

            let value = match value {
                Some(row) => Some((NodeKey::decode(&row.key)?, Node::decode(&row.value)?)),
                _ => None,
            };

            let mut node_key_and_node: Option<(NodeKey, LeafNode)> = None;

            if let Some((key, Node::Leaf(leaf_node))) = value {
                node_key_and_node.replace((key, leaf_node));
            }

            Ok(node_key_and_node)
        })
    }
}
