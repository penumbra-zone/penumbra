use std::collections::HashMap;

use anyhow::Result;
use jmt::storage::{LeafNode, Node, NodeKey, TreeReader};
use tracing::Span;

/// Snapshots maintain a point-in-time view of the underlying storage, suitable
/// for read-only access by multiple threads, i.e. RPC calls.
///
/// This is implemented as a wrapper around a [RocksDB snapshot](https://github.com/facebook/rocksdb/wiki/Snapshot)
/// with an associated JMT version number for the snapshot.
pub(crate) struct Snapshot {
    // TODO: the `'static` lifetime is a temporary hack and we'll need to find a workaround separately (tracked in #1512)
    rocksdb_snapshot: rocksdb::Snapshot<'static>,
    db: &'static rocksdb::DB,
    jmt_version: jmt::Version,
}

impl Snapshot {
    pub(crate) fn new(
        rocksdb_snapshot: rocksdb::Snapshot<'static>,
        jmt_version: jmt::Version,
        db: &'static rocksdb::DB,
    ) -> Self {
        Self {
            rocksdb_snapshot,
            jmt_version,
            db,
        }
    }

    pub fn get_raw(&self, key: String) -> Result<Option<Vec<u8>>> {
        self.rocksdb_snapshot.get(key).map_err(Into::into)
    }

    pub fn jmt_version(&self) -> jmt::Version {
        self.jmt_version
    }
}

/// A reader interface for rocksdb. NOTE: it is up to the caller to ensure consistency between the
/// rocksdb::DB handle and any write batches that may be applied through the writer interface.
impl TreeReader for Snapshot {
    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        let node_key = node_key.clone();

        let span = Span::current();

        let jmt_cf = self
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");
        let value = self
            .rocksdb_snapshot
            .get_cf(jmt_cf, &node_key.encode()?)?
            .map(|db_slice| Node::decode(&db_slice))
            .transpose()?;

        tracing::trace!(?node_key, ?value);
        Ok(value)
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>> {
        let jmt_cf = self
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");
        let mut iter = self.rocksdb_snapshot.raw_iterator_cf(jmt_cf);
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
}
