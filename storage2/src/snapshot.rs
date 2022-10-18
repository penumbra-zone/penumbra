use anyhow::Result;
use jmt::storage::{LeafNode, Node, NodeKey, TreeReader};

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

    /// Fetch a key from the JMT column family.
    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let jmt_cf = self
            .db
            .cf_handle("jmt")
            .expect("jmt column family not found");
        self.rocksdb_snapshot
            .get_cf(jmt_cf, key)
            .map_err(Into::into)
    }

    /// Fetch a key from the sidecar column family.
    pub fn get_sidecar(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let sidecar_cf = self
            .db
            .cf_handle("sidecar")
            .expect("sidecar column family not found");
        self.rocksdb_snapshot
            .get_cf(sidecar_cf, key)
            .map_err(Into::into)
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
        let node_key = node_key;
        tracing::trace!(?node_key);

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
        iter.seek_to_last();

        if iter.valid() {
            let node_key = NodeKey::decode(iter.key().unwrap())?;
            let node = Node::decode(iter.value().unwrap())?;

            if let Node::Leaf(leaf_node) = node {
                return Ok(Some((node_key, leaf_node)));
            }
        } else {
            // There are no keys in the database
        }

        Ok(None)
    }
}
