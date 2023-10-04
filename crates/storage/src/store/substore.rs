use std::sync::Arc;

use anyhow::Result;
use borsh::BorshDeserialize;
use jmt::{
    storage::{HasPreimage, LeafNode, Node, NodeKey, TreeReader},
    KeyHash,
};
use rocksdb::{ColumnFamily, IteratorMode, ReadOptions};

use crate::{
    snapshot::RocksDbSnapshot,
    storage::{DbNodeKey, VersionedKeyHash},
    Snapshot,
};

#[derive(Debug)]
pub struct SubstoreConfig {
    /// The prefix of the substore. If empty, it is the transparent top-level store config.
    pub prefix: String,
    /// name: "jmt"
    /// role: persists the logical structure of the JMT
    /// maps: `storage::DbNodeKey` to `jmt::Node`
    // note: `DbNodeKey` is a newtype around `NodeKey` that serialize the key
    // so that it maps to a lexicographical ordering with ascending jmt::Version.
    cf_jmt: String,
    /// jmt_keys: JMT key index.
    /// maps: key preimages to their keyhash.
    cf_jmt_keys: String,
    /// jmt_values: the actual values that JMT leaves point to.
    /// maps: KeyHash || BE(version) to an `Option<Vec<u8>>`
    cf_jmt_values: String,
    /// jmt_keys_by_keyhash: JMT keyhash index.
    /// maps: keyhashes to their preimage.
    cf_jmt_keys_by_keyhash: String,
    /// nonverifiable: auxiliary data that is not part of our merkle tree,
    /// and thus not strictly part of consensus (or verifiable).
    /// maps: arbitrary keys to arbitrary values.
    cf_nonverifiable: String,
    // This isn't part of the JMT, so the substore abstraction
    // isn't really necessary, but it's cleaner if we keep it
    // segmented by substore so that all reads can be dispached to a substore.
}

impl SubstoreConfig {
    pub fn new(prefix: String) -> Self {
        Self {
            cf_jmt: format!("substore-{}-jmt", prefix),
            cf_jmt_keys: format!("substore-{}-jmt-keys", prefix),
            cf_jmt_values: format!("substore-{}-jmt-values", prefix),
            cf_jmt_keys_by_keyhash: format!("substore-{}-jmt-keys-by-keyhash", prefix),
            cf_nonverifiable: format!("substore-{}-nonverifiable", prefix),
            prefix,
        }
    }

    pub fn transparent_store() -> Self {
        Self {
            // TODO: harmonize cf format throughout.
            cf_jmt: "jmt".to_string(),
            cf_jmt_keys: "jmt_keys".to_string(),
            cf_jmt_values: "jmt_values".to_string(),
            cf_jmt_keys_by_keyhash: "jmt_keys_by_keyhash".to_string(),
            cf_nonverifiable: "nonverifiable".to_string(),
            prefix: "".to_string(),
        }
    }

    /// Returns an iterator over all column families in this substore.
    /// This is verbose, but simple.
    pub fn columns(&self) -> impl Iterator<Item = &String> {
        std::iter::once(&self.cf_jmt)
            .chain(std::iter::once(&self.cf_jmt_keys))
            .chain(std::iter::once(&self.cf_jmt_values))
            .chain(std::iter::once(&self.cf_jmt_keys_by_keyhash))
            .chain(std::iter::once(&self.cf_nonverifiable))
    }
    pub fn cf_jmt<'s>(&self, snapshot: &'s Snapshot) -> &'s ColumnFamily {
        snapshot
            .0
            .db
            .cf_handle(self.cf_jmt.as_str())
            .expect("substore jmt column family not found")
    }

    pub fn cf_jmt_values<'s>(&self, snapshot: &'s Snapshot) -> &'s ColumnFamily {
        snapshot
            .0
            .db
            .cf_handle(self.cf_jmt_values.as_str())
            .expect("substore jmt-values column family not found")
    }

    pub fn cf_jmt_keys_by_keyhash<'s>(&self, snapshot: &'s Snapshot) -> &'s ColumnFamily {
        snapshot
            .0
            .db
            .cf_handle(self.cf_jmt_keys_by_keyhash.as_str())
            .expect("substore jmt-keys-by-keyhash column family not found")
    }

    pub fn cf_jmt_keys<'s>(&self, snapshot: &'s Snapshot) -> &'s ColumnFamily {
        snapshot
            .0
            .db
            .cf_handle(self.cf_jmt_keys.as_str())
            .expect("substore jmt-keys column family not found")
    }

    // TODO: we can use a `rocksdb::OptimisticTransactionDB` since we know that
    // our write load is not contentious (definitionally), and we can use make
    // writing to every substore atomic.
    pub fn _commit(&self, _changeset: ()) -> Result<()> {
        todo!("commit changeset to rocksdb")
    }
}

pub struct SubstoreSnapshot {
    pub config: Arc<SubstoreConfig>,
    pub snapshot: Snapshot,
}

impl SubstoreSnapshot {
    fn rocksdb_snapshot(&self) -> &RocksDbSnapshot {
        &self.snapshot.0.snapshot
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
        let jmt_values_cf = self.config.cf_jmt_values(&self.snapshot);

        // Prefix ranges exclude the upper bound in the iterator result.
        // This means that when requesting the largest possible version, there
        // is no way to specify a range that is inclusive of `u64::MAX`.
        if max_version == u64::MAX {
            let k = VersionedKeyHash {
                version: u64::MAX,
                key_hash,
            };

            if let Some(v) = self.rocksdb_snapshot().get_cf(jmt_values_cf, k.encode())? {
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
            self.rocksdb_snapshot()
                .iterator_cf_opt(jmt_values_cf, readopts, IteratorMode::End);

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

        let jmt_cf = self.config.cf_jmt(&self.snapshot);
        let value = self
            .rocksdb_snapshot()
            .get_cf(jmt_cf, db_node_key.encode()?)?
            .map(|db_slice| Node::try_from_slice(&db_slice))
            .transpose()?;

        tracing::trace!(?node_key, ?value);
        Ok(value)
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>> {
        let jmt_cf = self.config.cf_jmt(&self.snapshot);
        let mut iter = self.rocksdb_snapshot().raw_iterator_cf(jmt_cf);
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
        let jmt_keys_by_keyhash_cf = self.config.cf_jmt_keys_by_keyhash(&self.snapshot);

        Ok(self
            .rocksdb_snapshot()
            .get_cf(jmt_keys_by_keyhash_cf, key_hash.0)?)
    }
}
