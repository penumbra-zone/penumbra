use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use anyhow::Result;
use borsh::BorshDeserialize;
use jmt::{
    storage::{HasPreimage, LeafNode, Node, NodeKey, TreeReader},
    KeyHash, RootHash,
};
use rocksdb::{ColumnFamily, IteratorMode, ReadOptions};

use crate::{
    snapshot::RocksDbSnapshot,
    storage::{DbNodeKey, VersionedKeyHash},
};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct SubstoreConfig {
    /// The prefix of the substore. If empty, it is the root-level store config.
    pub prefix: String,
    /// name: "substore-{prefix}-jmt"
    /// role: persists the logical structure of the JMT
    /// maps: `storage::DbNodeKey` to `jmt::Node`
    // note: `DbNodeKey` is a newtype around `NodeKey` that serialize the key
    // so that it maps to a lexicographical ordering with ascending jmt::Version.
    cf_jmt: String,
    /// name: "susbstore-{prefix}-jmt-keys"
    /// role: JMT key index.
    /// maps: key preimages to their keyhash.
    cf_jmt_keys: String,
    /// name: "substore-{prefix}-jmt-values"
    /// role: stores the actual values that JMT leaves point to.
    /// maps: KeyHash || BE(version) to an `Option<Vec<u8>>`
    cf_jmt_values: String,
    /// name: "substore-{prefix}-jmt-keys-by-keyhash"
    /// role: index JMT keys by their keyhash.
    /// maps: keyhashes to their preimage.
    cf_jmt_keys_by_keyhash: String,
    /// name: "substore-{prefix}-nonverifiable"
    /// role: auxiliary data that is not part of our merkle tree, and thus not strictly
    /// part of consensus.
    /// maps: arbitrary keys to arbitrary values.
    cf_nonverifiable: String,
}

impl SubstoreConfig {
    pub fn new(prefix: impl ToString) -> Self {
        let prefix = prefix.to_string();
        Self {
            cf_jmt: format!("substore-{}-jmt", prefix),
            cf_jmt_keys: format!("substore-{}-jmt-keys", prefix),
            cf_jmt_values: format!("substore-{}-jmt-values", prefix),
            cf_jmt_keys_by_keyhash: format!("substore-{}-jmt-keys-by-keyhash", prefix),
            cf_nonverifiable: format!("substore-{}-nonverifiable", prefix),
            prefix,
        }
    }

    /// Returns an iterator over all column families in this substore.
    /// This is verbose, but very lightweight.
    pub fn columns(&self) -> impl Iterator<Item = &String> {
        std::iter::once(&self.cf_jmt)
            .chain(std::iter::once(&self.cf_jmt_keys))
            .chain(std::iter::once(&self.cf_jmt_values))
            .chain(std::iter::once(&self.cf_jmt_keys_by_keyhash))
            .chain(std::iter::once(&self.cf_nonverifiable))
    }

    pub fn cf_jmt<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt.as_str();
        db_handle.cf_handle(column).expect(&format!(
            "jmt column family not found for prefix: {}, substore: {}",
            column, self.prefix
        ))
    }

    pub fn cf_jmt_values<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_values.as_str();
        db_handle.cf_handle(column).expect(&format!(
            "jmt-values column family not found for prefix: {}, substore: {}",
            column, self.prefix
        ))
    }

    pub fn cf_jmt_keys_by_keyhash<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_keys_by_keyhash.as_str();
        db_handle.cf_handle(&column).expect(&format!(
            "jmt-keys-by-keyhash column family not found for prefix: {}, substore: {}",
            column, self.prefix
        ))
    }

    pub fn cf_jmt_keys<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_keys.as_str();
        db_handle.cf_handle(column).expect(&format!(
            "jmt-keys column family not found for prefix: {}, substore: {}",
            column, self.prefix
        ))
    }

    pub fn cf_nonverifiable<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_nonverifiable.as_str();
        db_handle.cf_handle(column).expect(&format!(
            "nonverifiable column family not found for prefix: {}, substore: {}",
            column, self.prefix
        ))
    }

    pub fn _commit<'s>(&self, _db_handle: &'s Arc<rocksdb::DB>) -> Result<RootHash> {
        todo!()
    }
}

impl Display for SubstoreConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubstoreConfig(prefix={})", self.prefix)
    }
}

pub struct SubstoreSnapshot {
    pub(crate) config: Arc<SubstoreConfig>,
    pub(crate) rocksdb_snapshot: Arc<RocksDbSnapshot>,
    pub(crate) version: jmt::Version,
    pub(crate) db: Arc<rocksdb::DB>,
}

impl SubstoreSnapshot {
    pub fn root_hash(&self) -> Result<crate::RootHash> {
        let version = self.version();
        let tree = jmt::Sha256Jmt::new(self);
        Ok(tree
            .get_root_hash_option(version)?
            .unwrap_or(jmt::RootHash([0; 32])))
    }

    pub fn version(&self) -> jmt::Version {
        self.version
    }

    /// Returns some value corresponding to the key, along with an ICS23 existence proof
    /// up to the current JMT root hash. If the key is not present, returns `None` and a
    /// non-existence proof.
    pub(crate) fn get_with_proof(
        &self,
        key: Vec<u8>,
    ) -> Result<(Option<Vec<u8>>, ics23::CommitmentProof)> {
        let version = self.version();
        let tree = jmt::Sha256Jmt::new(self);
        tree.get_with_ics23_proof(key, version)
    }

    /// Helper function used by `get_raw` and `prefix_raw`.
    ///
    /// Reads from the JMT will fail if the root is missing; this method
    /// special-cases the empty tree case so that reads on an empty tree just
    /// return None.
    pub fn get_jmt(&self, key: jmt::KeyHash) -> Result<Option<Vec<u8>>> {
        let tree = jmt::Sha256Jmt::new(self);
        match tree.get(key, self.version()) {
            Ok(Some(value)) => {
                tracing::trace!(version = ?self.version(), ?key, value = ?hex::encode(&value), "read from tree");
                Ok(Some(value))
            }
            Ok(None) => {
                tracing::trace!(version = ?self.version(), ?key, "key not found in tree");
                Ok(None)
            }
            // This allows for using the Overlay on an empty database without
            // errors We only skip the `MissingRootError` if the `version` is
            // `u64::MAX`, the pre-genesis version. Otherwise, a missing root
            // actually does indicate a problem.
            Err(e)
                if e.downcast_ref::<jmt::MissingRootError>().is_some()
                    && self.version() == u64::MAX =>
            {
                tracing::trace!(version = ?self.version(), "no data available at this version");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

impl TreeReader for SubstoreSnapshot {
    /// Gets a value by identifier, returning the newest value whose version is *less than or
    /// equal to* the specified version.  Returns `None` if the value does not exist.
    fn get_value_option(
        &self,
        max_version: jmt::Version,
        key_hash: KeyHash,
    ) -> Result<Option<jmt::OwnedValue>> {
        let cf_jmt_values = self.config.cf_jmt_values(&self.db);

        // Prefix ranges exclude the upper bound in the iterator result.
        // This means that when requesting the largest possible version, there
        // is no way to specify a range that is inclusive of `u64::MAX`.
        if max_version == u64::MAX {
            let k = VersionedKeyHash {
                version: u64::MAX,
                key_hash,
            };

            if let Some(v) = self.rocksdb_snapshot.get_cf(cf_jmt_values, k.encode())? {
                let maybe_value: Option<Vec<u8>> = BorshDeserialize::try_from_slice(v.as_ref())?;
                return Ok(maybe_value);
            }
        }

        let mut lower_bound = key_hash.0.to_vec();
        lower_bound.extend_from_slice(&0u64.to_be_bytes());

        let mut upper_bound = key_hash.0.to_vec();
        // The upper bound is excluded from the iteration results.
        upper_bound.extend_from_slice(&(max_version.saturating_add(1)).to_be_bytes());

        let mut readopts = ReadOptions::default();
        readopts.set_iterate_lower_bound(lower_bound);
        readopts.set_iterate_upper_bound(upper_bound);
        let mut iterator =
            self.rocksdb_snapshot
                .iterator_cf_opt(cf_jmt_values, readopts, IteratorMode::End);

        let Some(tuple) = iterator.next() else {
            return Ok(None);
        };

        let (_key, v) = tuple?;
        let maybe_value = BorshDeserialize::try_from_slice(v.as_ref())?;
        Ok(maybe_value)
    }

    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        let node_key = node_key;
        let db_node_key = DbNodeKey::from(node_key.clone());
        tracing::trace!(?node_key);

        let cf_jmt = self.config.cf_jmt(&self.db);
        let value = self
            .rocksdb_snapshot
            .get_cf(cf_jmt, db_node_key.encode()?)?
            .map(|db_slice| Node::try_from_slice(&db_slice))
            .transpose()?;

        tracing::trace!(?node_key, ?value);
        Ok(value)
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>> {
        let cf_jmt = self.config.cf_jmt(&self.db);
        let mut iter = self.rocksdb_snapshot.raw_iterator_cf(cf_jmt);
        iter.seek_to_last();

        if iter.valid() {
            let node_key =
                DbNodeKey::decode(iter.key().expect("all DB entries should have a key"))?
                    .into_inner();
            let node =
                Node::try_from_slice(iter.value().expect("all DB entries should have a value"))?;

            if let Node::Leaf(leaf_node) = node {
                return Ok(Some((node_key, leaf_node)));
            }
        } else {
            // There are no keys in the database
        }

        Ok(None)
    }
}

impl HasPreimage for SubstoreSnapshot {
    fn preimage(&self, key_hash: KeyHash) -> Result<Option<Vec<u8>>> {
        let cf_jmt_keys_by_keyhash = self.config.cf_jmt_keys_by_keyhash(&self.db);

        Ok(self
            .rocksdb_snapshot
            .get_cf(cf_jmt_keys_by_keyhash, key_hash.0)?)
    }
}
