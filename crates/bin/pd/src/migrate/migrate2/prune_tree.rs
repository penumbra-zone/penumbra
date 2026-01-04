//! A migration that prunes extraneous nodes from a Jellyfish Merkle Tree.
//!
//! **Context**
//! The Jellyfish Merkle Tree (JMT) is a **versioned** sparse merkle tree data structure.
//! Every update to the tree bumps a monotonic version counter. The branching factor for
//! the JMT is 16. Each key insert is hashed into a 256 bits path key.
//! k := H(key) e.g, 0x0123456789abcdef, where each nibble is a conceptual level in the tree.
//! It is conceptual because the tree is compact so depth can be lower. Nodes are addressed by
//! their version and nibble path. The former allows storage and abstraction layers to improve
//! locality of caches and SSD layers.
//!
//! By default, historical state is preserved which allows full nodes to answer historical
//! queries, and generate proofs at any version. In practice, there is no code using this
//! conceptual feature. It just takes up storage.
//!
//! **Amplification**
//! Without pruning, stale nodes are preserved across writes. This allows a tree
//! to answer historical queries about arbitrary versions, at the cost of storage
//! and slower queries on cold rocksdb seeks.
//!
//! The amplification from those extranodes is superlinear in the number of inserts.
//! The quick intuition is that each key update results in a new leaf and a stack of
//! ancestors. Without pruning, those ancestors nodes are never reclaimed. Therefore,
//! each update results in one stack of node per update.
//!
//! How many nodes are in that stack? It varies, but it will be about the height of
//! the compact tree which scales with the logarithm of the # of keys in the tree.
//! that gives an amplification of N * log M, where N is the number of updates, and
//! M the number of keys. We don't account for overlapping ancestor nodes since it
//! asymptotically resolves to a logarithm factor anyway.
//!
//! **Implementation**
//! JMT pruning is implemented as a [`Migration`](super::Migration) that overwrites
//! default impls to no-op any genesis, or application state preparation. Since JMT
//! pruning MUST NOT be consensus breaking, it does not require a hard-fork or any
//! coordination. It is an entire local decision transparent to the rest of the
//! network.
//!
//! Defining it as a local migration has two benefits, JMT pruning:
//! 1. .. is just another offline state transition, and reuses existing tooling.
//! 2. .. can be composed by other migration scripts, or used on its own.
//!
//! In order to prune storage, we:
//! 1. Create a fresh database
//! 2. Stream (key, value) pairs in chunks using JellyfishMerkleIterator
//! 3. For each chunk, generate a range proof from the old tree
//! 4. Feed chunks + proofs to JellyfishMerkleRestore (validates incrementally)
//! 5. finish() verifies the final root hash matches the original
//! 6. Copy auxiliary column families from old to new database
//! 7. Swap database directories atomically
use std::{path::{Path, PathBuf}, sync::Arc};

use anyhow::Result;
use cnidarium::{DbNodeKey, StateDelta, Storage, SubstoreConfig, SubstoreSnapshot, VersionedKeyHash};
use jmt::{
    restore::{JellyfishMerkleRestore, StateSnapshotReceiver},
    storage::{Node, NodeBatch, NodeKey, TreeReader, TreeWriter},
    JellyfishMerkleIterator, JellyfishMerkleTree, KeyHash, OwnedValue, RootHash,
};
use penumbra_sdk_app::SUBSTORE_PREFIXES;
use rocksdb::DB;

use super::Migration;

/// Copy all entries from one column family to another, preserving key/value bytes exactly.
/// Returns the number of entries copied.
fn copy_column_family(old_db: &DB, new_db: &DB, cf_name: &str) -> Result<u64> {
    let old_cf = old_db
        .cf_handle(cf_name)
        .ok_or_else(|| anyhow::anyhow!("column family '{}' not found in old database", cf_name))?;
    let new_cf = new_db
        .cf_handle(cf_name)
        .ok_or_else(|| anyhow::anyhow!("column family '{}' not found in new database", cf_name))?;

    let mut count = 0u64;
    let mut batch = rocksdb::WriteBatch::default();

    let mut iter = old_db.raw_iterator_cf(old_cf);
    iter.seek_to_first();

    while iter.valid() {
        if let (Some(key), Some(value)) = (iter.key(), iter.value()) {
            batch.put_cf(new_cf, key, value);
            count += 1;

            // Write in batches to avoid memory bloat
            if count % 10_000 == 0 {
                new_db.write(std::mem::take(&mut batch))?;
            }
        }
        iter.next();
    }

    // Write any remaining entries
    if !batch.is_empty() {
        new_db.write(batch)?;
    }

    Ok(count)
}

/// Default chunk size for streaming key-value pairs to the restore process.
/// Larger chunks are more efficient but use more memory.
/// Each chunk requires a range proof to be generated.
/// Can be overridden via the PRUNE_CHUNK_SIZE environment variable.
const DEFAULT_CHUNK_SIZE: usize = 100_000;

/// A TreeWriter/TreeReader implementation that accesses RocksDB directly.
/// Used during pruning to write and verify the compacted JMT.
struct PruningTreeStore {
    db: Arc<DB>,
    config: Arc<SubstoreConfig>,
}

impl PruningTreeStore {
    fn new(db: Arc<DB>, config: Arc<SubstoreConfig>) -> Self {
        Self { db, config }
    }
}

impl TreeWriter for PruningTreeStore {
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()> {
        let cf_jmt = self.config.cf_jmt(&self.db);
        let cf_jmt_values = self.config.cf_jmt_values(&self.db);

        let mut batch = rocksdb::WriteBatch::default();

        // Write nodes
        for (node_key, node) in node_batch.nodes() {
            let key_bytes = DbNodeKey::encode_from_node_key(node_key)?;
            let value_bytes = borsh::to_vec(node)?;
            batch.put_cf(cf_jmt, key_bytes, value_bytes);
        }

        // Write values
        for ((version, key_hash), some_value) in node_batch.values() {
            let key_bytes = VersionedKeyHash::encode_from_keyhash(key_hash, version);
            let value_bytes = borsh::to_vec(some_value)?;
            batch.put_cf(cf_jmt_values, key_bytes, value_bytes);
        }

        self.db.write(batch)?;
        Ok(())
    }
}

impl TreeReader for PruningTreeStore {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        let cf_jmt = self.config.cf_jmt(&self.db);
        let key_bytes = DbNodeKey::encode_from_node_key(node_key)?;

        match self.db.get_pinned_cf(cf_jmt, &key_bytes)? {
            Some(bytes) => {
                let node: Node = borsh::from_slice(&bytes)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    fn get_value_option(
        &self,
        max_version: jmt::Version,
        key_hash: KeyHash,
    ) -> Result<Option<OwnedValue>> {
        let cf_jmt_values = self.config.cf_jmt_values(&self.db);

        // Look for the value at exactly max_version
        let key_bytes = VersionedKeyHash::encode_from_keyhash(&key_hash, &max_version);
        match self.db.get_pinned_cf(cf_jmt_values, &key_bytes)? {
            Some(bytes) => {
                let value: Option<Vec<u8>> = borsh::from_slice(&bytes)?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, jmt::storage::LeafNode)>> {
        // Not needed for our use case
        Ok(None)
    }
}

/// A state migration that prunes the extraneous nodes from a Jellyfish Merkle Tree.
pub struct JellyfishTreePruner;

impl JellyfishTreePruner {
    /// Checks if the JMT needs pruning by counting nodes at versions other than current_version.
    /// Returns (nodes_at_current, nodes_at_other_versions).
    fn count_nodes_by_version(
        &self,
        db: &Arc<DB>,
        config: &SubstoreConfig,
        current_version: jmt::Version,
    ) -> Result<(u64, u64)> {
        let cf_jmt = config.cf_jmt(db);

        let mut at_current = 0u64;
        let mut at_other = 0u64;

        let mut iter = db.raw_iterator_cf(cf_jmt);
        iter.seek_to_first();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if key.len() >= 8 {
                    let version_bytes: [u8; 8] = key[0..8].try_into().expect("checked length");
                    let version = u64::from_be_bytes(version_bytes);
                    if version == current_version {
                        at_current += 1;
                    } else {
                        at_other += 1;
                    }
                }
            }
            iter.next();
        }

        Ok((at_current, at_other))
    }
}

impl Migration for JellyfishTreePruner {
    fn name(&self) -> &'static str {
        "jmt-pruning"
    }

    fn target_app_version(&self) -> Option<u64> {
        // Non-consensus-breaking, no version bump needed
        None
    }

    async fn migrate(
        &self,
        pd_home: &PathBuf,
        _comet_home: Option<&PathBuf>,
    ) -> Result<(RootHash, u64)> {
        let rocksdb_dir = pd_home.join("rocksdb");
        let rocksdb_new = pd_home.join("rocksdb_new");
        let rocksdb_old = pd_home.join("rocksdb_old");

        // Log initial directory size
        let initial_size = dir_size(&rocksdb_dir);
        tracing::info!(initial_size_bytes = initial_size, "rocksdb directory size before pruning");

        let storage = Storage::load(rocksdb_dir.clone(), SUBSTORE_PREFIXES.clone()).await?;
        let snapshot = storage.latest_snapshot();
        let original_root_hash = snapshot.root_hash().await?;
        let version = snapshot.version();

        tracing::info!(?original_root_hash, version, "starting JMT pruning");

        let db = storage.db();
        let rocksdb_snapshot = snapshot.rocksdb_snapshot();
        let main_config = Arc::new(SubstoreConfig::new(""));

        // Check if pruning is needed
        let (nodes_at_current, nodes_at_other) =
            self.count_nodes_by_version(&db, &main_config, version)?;
        tracing::info!(nodes_at_current, nodes_at_other, "checked node versions");

        if nodes_at_other == 0 {
            tracing::info!("all nodes already at current version, no pruning needed");
            drop(rocksdb_snapshot);
            drop(snapshot);
            drop(db);
            storage.release().await;

            // Clean up LOG.old files created by opening the database
            for entry in std::fs::read_dir(&rocksdb_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("LOG.old") {
                        std::fs::remove_file(entry.path())?;
                    }
                }
            }

            return Ok((original_root_hash, version));
        }

        tracing::info!(nodes_at_other, "found historical nodes to prune");

        /* *************** create fresh database and stream chunks *************** */
        if rocksdb_new.exists() {
            // Clean up any leftover directories from previous failed runs
            std::fs::remove_dir_all(&rocksdb_new)?;
        }

        if rocksdb_old.exists() {
            std::fs::remove_dir_all(&rocksdb_old)?;
        }

        tracing::info!("creating fresh database at {:?}", rocksdb_new);
        let new_storage = Storage::load(rocksdb_new.clone(), SUBSTORE_PREFIXES.clone()).await?;
        let new_db = new_storage.db();
        let tree_store = Arc::new(PruningTreeStore::new(new_db.clone(), main_config.clone()));

        // Create snapshot for iteration and proof generation
        let substore_snapshot =
            SubstoreSnapshot::new(main_config.clone(), rocksdb_snapshot.clone(), version, db.clone());
        let substore_snapshot_arc = Arc::new(substore_snapshot);

        // Create JMT for range proof generation (read from old database)
        let old_tree = JellyfishMerkleTree::<_, sha2::Sha256>::new(substore_snapshot_arc.as_ref());

        // Read chunk size from environment or use default
        let chunk_size: usize = std::env::var("PRUNE_CHUNK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CHUNK_SIZE);

        // Create restore instance (write to new database)
        // Note: new() requires range proofs for each chunk, providing incremental verification.
        // For trusted sources, new_overwrite() could skip proofs for better throughput.
        tracing::info!("initializing JellyfishMerkleRestore at version {}", version);
        let mut restore = JellyfishMerkleRestore::<sha2::Sha256>::new(
            tree_store.clone(),
            version,
            original_root_hash,
        )?;

        // Stream chunks from old database to new database
        tracing::info!(chunk_size, "streaming key-value pairs with range proofs");
        let iter = JellyfishMerkleIterator::new(
            substore_snapshot_arc.clone(),
            version,
            KeyHash([0u8; 32]),
        )?;

        let mut chunk: Vec<(KeyHash, OwnedValue)> = Vec::with_capacity(chunk_size);
        let mut total_count = 0u64;
        let mut chunk_count = 0u64;

        for result in iter {
            let (key_hash, value) = result?;
            chunk.push((key_hash, value));
            total_count += 1;

            if chunk.len() == chunk_size { // chop chop
                // Generate range proof for the rightmost key in this chunk
                let rightmost_key = chunk.last().unwrap().0;
                let proof = old_tree.get_range_proof(rightmost_key, version)?;
                let chunk_data: Vec<_> = chunk.drain(..).collect();
                restore.add_chunk(chunk_data, proof)?;
                chunk_count += 1;
                tracing::debug!(chunk_count, total_count, "processed chunk");
            }
        }

        // We process the remianing entries.
        if !chunk.is_empty() {
            let rightmost_key = chunk.last().unwrap().0;
            let proof = old_tree.get_range_proof(rightmost_key, version)?;
            restore.add_chunk(chunk, proof)?;
            chunk_count += 1;
        }

        tracing::info!(total_count, chunk_count, "streamed all key-value pairs (root hash verified via range proofs)");

        // Finish restore , flushing buffered data
        restore.finish()?;

        // Drop the snapshot but keep the db handle for copying CFs later
        drop(substore_snapshot_arc);
        drop(rocksdb_snapshot);
        drop(snapshot);

        // Verify all nodes are at current version
        let (nodes_at_current_post, nodes_at_other_post) =
            self.count_nodes_by_version(&new_db, &main_config, version)?;
        tracing::info!(
            nodes_at_current_post,
            nodes_at_other_post,
            "post-rebuild node count verification"
        );

        /* **************** copy auxiliary and substore column families **************** */
        tracing::info!("copying auxiliary column families from old database");

        let main_aux_cfs = [
            "config",
            "substore--jmt-keys",
            "substore--jmt-keys-by-keyhash",
            "substore--jmt-values",
            "substore--nonverifiable",
        ];
        for cf_name in main_aux_cfs {
            let count = copy_column_family(&db, &new_db, cf_name)?;
            tracing::info!(cf_name, count, "copied column family");
        }

        /* **************** copy all substores **************** */
        tracing::info!("copying substore column families");
        for prefix in SUBSTORE_PREFIXES.iter() {
            let substore_cfs = [
                format!("substore-{}-jmt", prefix),
                format!("substore-{}-jmt-keys", prefix),
                format!("substore-{}-jmt-values", prefix),
                format!("substore-{}-jmt-keys-by-keyhash", prefix),
                format!("substore-{}-nonverifiable", prefix),
            ];
            for cf_name in substore_cfs {
                let count = copy_column_family(&db, &new_db, &cf_name)?;
                tracing::info!(cf_name, count, "copied column family");
            }
        }

        drop(new_db);
        drop(db);
        new_storage.release().await;
        storage.release().await;
        tracing::info!("closed both databases");

        // Switch unpruned and pruned databases
        tracing::info!("swapping database directories");
        std::fs::rename(&rocksdb_dir, &rocksdb_old)?;
        std::fs::rename(&rocksdb_new, &rocksdb_dir)?;

        // Delete old database
        tracing::info!("removing old database");
        std::fs::remove_dir_all(&rocksdb_old)?;

        // Clean up RocksDB LOG.old files from the new database
        // (RocksDB creates these on each open, they can accumulate to megabytes)
        for entry in std::fs::read_dir(&rocksdb_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with("LOG.old") {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        let final_size = dir_size(&rocksdb_dir);
        let saved = initial_size.saturating_sub(final_size);
        tracing::info!(
            "pruning complete: saved {:.1} GB ({:.1} GB -> {:.1} GB)",
            saved as f64 / 1e9,
            initial_size as f64 / 1e9,
            final_size as f64 / 1e9,
        );

        Ok((original_root_hash, version))
    }

    async fn migrate_inner(&self, _delta: &mut StateDelta<cnidarium::Snapshot>) -> Result<()> {
        // Unused - we override migrate() directly
        Ok(())
    }

    async fn complete(
        &self,
        _pd_home: &PathBuf,
        _comet_home: Option<&PathBuf>,
        _post_upgrade_root_hash: jmt::RootHash,
        _post_upgrade_height: u64,
        _genesis_start: Option<tendermint::time::Time>,
    ) -> Result<()> {
        // No-op: no genesis needed for pruning
        Ok(())
    }
}

/// Helper to calculate directory size recursively.
fn dir_size(path: &Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path);
            } else if let Ok(meta) = path.metadata() {
                size += meta.len();
            }
        }
    }
    size
}
