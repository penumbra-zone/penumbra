use anyhow::Result;
use futures::future::BoxFuture;
use jmt::{
    hash::HashValue,
    node_type::{LeafNode, Node, NodeKey},
    NodeBatch, TreeReaderAsync, TreeWriterAsync, Value,
};
use sqlx::{query, Postgres};
use tracing::instrument;

use crate::State;

pub enum Key {
    NoteCommitmentAnchor,
}

impl Key {
    pub fn hash(self) -> HashValue {
        match self {
            Key::NoteCommitmentAnchor => HashValue::sha3_256_of(b"nct"),
        }
    }
}

/// Wrapper struct used to implement [`jmt::TreeWriterAsync`] for a Postgres
/// transaction, without violating the orphan rules.
pub struct DbTx<'conn, 'tx>(pub &'tx mut sqlx::Transaction<'conn, Postgres>);

impl<'conn, 'tx, V> TreeWriterAsync<V> for DbTx<'conn, 'tx>
where
    V: Value,
{
    /// Writes a node batch into storage.
    #[instrument(skip(self, node_batch))]
    fn write_node_batch<'future, 'a: 'future, 'n: 'future>(
        &'a mut self,
        node_batch: &'n NodeBatch<V>,
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

impl<V: Value> TreeReaderAsync<V> for State {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    #[instrument(skip(self))]
    fn get_node_option<'future, 'a: 'future, 'n: 'future>(
        &'a self,
        node_key: &'n NodeKey,
    ) -> BoxFuture<'future, Result<Option<Node<V>>>> {
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
    #[instrument(skip(self))]
    fn get_rightmost_leaf<'future, 'a: 'future>(
        &'a self,
    ) -> BoxFuture<'future, Result<Option<(NodeKey, LeafNode<V>)>>> {
        Box::pin(async {
            let mut conn = self.pool.acquire().await?;

            let value = query!(r#"SELECT key, value FROM jmt ORDER BY key DESC LIMIT 1"#)
                .fetch_optional(&mut conn)
                .await?;

            let value = match value {
                Some(row) => Some((NodeKey::decode(&row.key)?, Node::decode(&row.value)?)),
                _ => None,
            };

            let mut node_key_and_node: Option<(NodeKey, LeafNode<V>)> = None;

            if let Some((key, Node::Leaf(leaf_node))) = value {
                if node_key_and_node.is_none()
                    || leaf_node.account_key() > node_key_and_node.as_ref().unwrap().1.account_key()
                {
                    node_key_and_node.replace((key, leaf_node));
                }
            }

            Ok(node_key_and_node)
        })
    }
}
