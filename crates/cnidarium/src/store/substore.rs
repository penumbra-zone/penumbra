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
use tracing::Span;

use crate::{snapshot::RocksDbSnapshot, Cache};

use jmt::storage::TreeWriter;

/// Specifies the configuration of a substore, which is a prefixed subset of
/// the main store with its own merkle tree, nonverifiable data, preimage index, etc.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SubstoreConfig {
    /// The prefix of the substore. If empty, it is the root-level store config.
    pub prefix: String,
    /// The prefix of the substore including the trailing slash.
    pub prefix_with_delimiter: String,
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
            prefix_with_delimiter: format!("{}/", prefix),
            prefix,
        }
    }

    /// Returns an iterator over all column families in this substore.
    /// Note(erwan): This is verbose, but very lightweight.
    pub fn columns(&self) -> impl Iterator<Item = &String> {
        std::iter::once(&self.cf_jmt)
            .chain(std::iter::once(&self.cf_jmt_keys))
            .chain(std::iter::once(&self.cf_jmt_values))
            .chain(std::iter::once(&self.cf_jmt_keys_by_keyhash))
            .chain(std::iter::once(&self.cf_nonverifiable))
    }

    pub fn cf_jmt<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt.as_str();
        db_handle.cf_handle(column).unwrap_or_else(|| {
            panic!(
                "jmt column family not found for prefix: {}, substore: {}",
                column, self.prefix
            )
        })
    }

    pub fn cf_jmt_values<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_values.as_str();
        db_handle.cf_handle(column).unwrap_or_else(|| {
            panic!(
                "jmt-values column family not found for prefix: {}, substore: {}",
                column, self.prefix
            )
        })
    }

    pub fn cf_jmt_keys_by_keyhash<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_keys_by_keyhash.as_str();
        db_handle.cf_handle(column).unwrap_or_else(|| {
            panic!(
                "jmt-keys-by-keyhash column family not found for prefix: {}, substore: {}",
                column, self.prefix
            )
        })
    }

    pub fn cf_jmt_keys<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_jmt_keys.as_str();
        db_handle.cf_handle(column).unwrap_or_else(|| {
            panic!(
                "jmt-keys column family not found for prefix: {}, substore: {}",
                column, self.prefix
            )
        })
    }

    pub fn cf_nonverifiable<'s>(&self, db_handle: &'s Arc<rocksdb::DB>) -> &'s ColumnFamily {
        let column = self.cf_nonverifiable.as_str();
        db_handle.cf_handle(column).unwrap_or_else(|| {
            panic!(
                "nonverifiable column family not found for prefix: {}, substore: {}",
                column, self.prefix
            )
        })
    }

    pub fn latest_version_from_db(
        &self,
        db_handle: &Arc<rocksdb::DB>,
    ) -> Result<Option<jmt::Version>> {
        Ok(self
            .get_rightmost_leaf_from_db(db_handle)?
            .map(|(node_key, _)| node_key.version()))
    }

    pub fn latest_version_from_snapshot(
        &self,
        db_handle: &Arc<rocksdb::DB>,
        snapshot: &RocksDbSnapshot,
    ) -> Result<Option<jmt::Version>> {
        Ok(self
            .get_rightmost_leaf_from_snapshot(db_handle, snapshot)?
            .map(|(node_key, _)| node_key.version()))
    }

    // TODO(erwan): having two different implementations of this is a bit weird and should
    // be refactored, or remodeled. The DB version is only used during initialization, before
    // a `Snapshot` is available.
    fn get_rightmost_leaf_from_db(
        &self,
        db_handle: &Arc<rocksdb::DB>,
    ) -> Result<Option<(NodeKey, LeafNode)>> {
        let cf_jmt = self.cf_jmt(db_handle);
        let mut iter = db_handle.raw_iterator_cf(cf_jmt);
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

    fn get_rightmost_leaf_from_snapshot(
        &self,
        db_handle: &Arc<rocksdb::DB>,
        snapshot: &RocksDbSnapshot,
    ) -> Result<Option<(NodeKey, LeafNode)>> {
        let cf_jmt = self.cf_jmt(db_handle);
        let mut iter = snapshot.iterator_cf(cf_jmt, IteratorMode::End);
        let Some((raw_key, raw_value)) = iter.next().transpose()? else {
            return Ok(None);
        };

        let node_key = DbNodeKey::decode(&raw_key)?.into_inner();
        let Node::Leaf(leaf) = Node::try_from_slice(&raw_value)? else {
            return Ok(None);
        };
        Ok(Some((node_key, leaf)))
    }
}

impl Display for SubstoreConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubstoreConfig(prefix={})", self.prefix)
    }
}

/// A read-only view into a substore at a specific state version.
///
/// A [`SubstoreSnapshot`] is lightweight and cheap to create, it can be
/// instantiated on-demand when a read-only view of a substore's state is
/// needed.
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
                tracing::trace!(substore = ?self.config.prefix, version = ?self.version(), ?key, value = ?hex::encode(&value), "read from tree");
                Ok(Some(value))
            }
            Ok(None) => {
                tracing::trace!(substore = ?self.config.prefix, version = ?self.version(), ?key, "key not found in tree");
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
                tracing::trace!(substore = ?self.config.prefix, version = ?self.version(), "no data available at this version");
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

pub struct SubstoreStorage {
    pub(crate) substore_snapshot: SubstoreSnapshot,
}

impl SubstoreStorage {
    pub async fn commit(
        self,
        cache: Cache,
        mut write_batch: rocksdb::WriteBatch,
        write_version: jmt::Version,
        perform_migration: bool,
    ) -> Result<(RootHash, rocksdb::WriteBatch)> {
        let span = Span::current();

        tokio::task
                ::spawn_blocking(move || {
                    span.in_scope(|| {
                        let jmt = jmt::Sha256Jmt::new(&self.substore_snapshot);
                        let unwritten_changes: Vec<_> = cache
                            .unwritten_changes
                            .into_iter()
                            .map(|(key, some_value)| (KeyHash::with::<sha2::Sha256>(&key), key, some_value))
                            .collect();

                        let cf_jmt_keys = self.substore_snapshot.config.cf_jmt_keys(&self.substore_snapshot.db);
                        let cf_jmt_keys_by_keyhash = self.substore_snapshot.config.cf_jmt_keys_by_keyhash(&self.substore_snapshot.db);
                        let cf_jmt = self.substore_snapshot.config.cf_jmt(&self.substore_snapshot.db);
                        let cf_jmt_values = self.substore_snapshot.config.cf_jmt_values(&self.substore_snapshot.db);

                        /* Keyhash and pre-image indices */
                        for (keyhash, key_preimage, value) in unwritten_changes.iter() {
                            match value {
                                Some(_) => { /* Key inserted, or updated, so we add it to the keyhash index */
                                    write_batch.put_cf(cf_jmt_keys, key_preimage, keyhash.0);
                                        write_batch
                                        .put_cf(cf_jmt_keys_by_keyhash, keyhash.0, key_preimage)
                                }
                                None => { /* Key deleted, so we delete it from the preimage and keyhash index entries */
                                    write_batch.delete_cf(cf_jmt_keys, key_preimage);
                                    write_batch.delete_cf(cf_jmt_keys_by_keyhash, keyhash.0);
                                }
                            };
                        }

                        // We only track the keyhash and possible values; at the time of writing,
                        // `rustfmt` panics on inlining the closure, so we use a helper function to skip the key.
                        let skip_key = |(keyhash, _key, some_value)| (keyhash, some_value);

                        let (root_hash, batch) = if perform_migration {
                            jmt.append_value_set(unwritten_changes.into_iter().map(skip_key), write_version)?
                        } else {
                            jmt.put_value_set(unwritten_changes.into_iter().map(skip_key), write_version)?
                        };

                        /* JMT nodes and values */
                        for (node_key, node) in batch.node_batch.nodes() {
                            let db_node_key_bytes= DbNodeKey::encode_from_node_key(node_key)?;
                            let value_bytes = borsh::to_vec(node)?;
                            tracing::trace!(?db_node_key_bytes, value_bytes = ?hex::encode(&value_bytes));
                            write_batch.put_cf(cf_jmt, db_node_key_bytes, value_bytes);
                        }


                        for ((version, key_hash), some_value) in batch.node_batch.values() {
                            let key_bytes = VersionedKeyHash::encode_from_keyhash(key_hash, version);
                            let value_bytes = borsh::to_vec(some_value)?;
                            tracing::trace!(?key_bytes, value_bytes = ?hex::encode(&value_bytes));
                            write_batch.put_cf(cf_jmt_values, key_bytes, value_bytes);
                        }

                        tracing::trace!(?root_hash, "accumulated node changes in the write batch");


                        for (k, v) in cache.nonverifiable_changes.into_iter() {
                            let cf_nonverifiable = self.substore_snapshot.config.cf_nonverifiable(&self.substore_snapshot.db);
                            match v {
                                Some(v) => {
                                    tracing::trace!(key = ?crate::EscapedByteSlice(&k), value = ?crate::EscapedByteSlice(&v), "put nonverifiable key");
                                    write_batch.put_cf(cf_nonverifiable, k, &v);
                                }
                                None => {
                                    write_batch.delete_cf(cf_nonverifiable, k);
                                }
                            };
                        }

                        Ok((root_hash, write_batch))
                    })
                })
                .await?
    }
}

impl TreeWriter for SubstoreStorage {
    fn write_node_batch(&self, _node_batch: &jmt::storage::NodeBatch) -> Result<()> {
        // The "write"-part of the `TreeReader + TreeWriter` jmt architecture does not work
        // well with a deferred write strategy.
        // What we would like to do is to accumulate the changes in a write batch, and then commit
        // them all at once. This isn't possible to do easily because the `TreeWriter` trait
        // rightfully does not expose RocksDB-specific types in its API.
        //
        // The alternative is to use interior mutability but the semantics become
        // so implementation specific that we lose the benefits of the trait abstraction.
        unimplemented!("We inline the tree writing logic in the `commit` method")
    }
}

/// An ordered node key is a node key that is encoded in a way that
/// preserves the order of the node keys in the database.
pub struct DbNodeKey(pub NodeKey);

impl DbNodeKey {
    pub fn from(node_key: NodeKey) -> Self {
        DbNodeKey(node_key)
    }

    pub fn into_inner(self) -> NodeKey {
        self.0
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Self::encode_from_node_key(&self.0)
    }

    pub fn encode_from_node_key(node_key: &NodeKey) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&node_key.version().to_be_bytes()); // encode version as big-endian
        let rest = borsh::to_vec(node_key)?;
        bytes.extend_from_slice(&rest);
        Ok(bytes)
    }

    pub fn decode(bytes: impl AsRef<[u8]>) -> Result<Self> {
        if bytes.as_ref().len() < 8 {
            anyhow::bail!("byte slice is too short")
        }
        // Ignore the bytes that encode the version
        let node_key_slice = bytes.as_ref()[8..].to_vec();
        let node_key = borsh::BorshDeserialize::try_from_slice(&node_key_slice)?;
        Ok(DbNodeKey(node_key))
    }
}

/// Represent a JMT key hash at a specific `jmt::Version`
/// This is used to index the JMT values in RocksDB.
#[derive(Clone, Debug)]
pub struct VersionedKeyHash {
    pub key_hash: KeyHash,
    pub version: jmt::Version,
}

impl VersionedKeyHash {
    pub fn encode(&self) -> Vec<u8> {
        VersionedKeyHash::encode_from_keyhash(&self.key_hash, &self.version)
    }

    pub fn encode_from_keyhash(key_hash: &KeyHash, version: &jmt::Version) -> Vec<u8> {
        let mut buf: Vec<u8> = key_hash.0.to_vec();
        buf.extend_from_slice(&version.to_be_bytes());
        buf
    }

    #[allow(dead_code)]
    pub fn decode(buf: Vec<u8>) -> Result<Self> {
        if buf.len() != 40 {
            Err(anyhow::anyhow!(
                "could not decode buffer into VersionedKey (invalid size)"
            ))
        } else {
            let raw_key_hash: [u8; 32] = buf[0..32]
                .try_into()
                .expect("buffer is at least 40 bytes wide");
            let key_hash = KeyHash(raw_key_hash);

            let raw_version: [u8; 8] = buf[32..40]
                .try_into()
                .expect("buffer is at least 40 bytes wide");
            let version: u64 = u64::from_be_bytes(raw_version);

            Ok(VersionedKeyHash { version, key_hash })
        }
    }
}
