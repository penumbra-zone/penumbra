//! A migration that prunes extraneous nodes from a Jellyfish Merkle Tree.
//!
//! This migration uses cnidarium's `prune_substore` to stream key-value pairs
//! with range proof verification, then handles pd-specific concerns like
//! directory swapping and substore copying.

use std::path::{Path, PathBuf};

use anyhow::Result;
use cnidarium::{prune_main_substore, PruneConfig, StateDelta, Storage};
use jmt::RootHash;
use penumbra_sdk_app::SUBSTORE_PREFIXES;
use rocksdb::DB;

use super::Migration;

/// Copy all entries from one column family to another, preserving key/value bytes exactly.
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

            if count % 10_000 == 0 {
                new_db.write(std::mem::take(&mut batch))?;
            }
        }
        iter.next();
    }

    if !batch.is_empty() {
        new_db.write(batch)?;
    }

    Ok(count)
}

/// A state migration that prunes the extraneous nodes from a Jellyfish Merkle Tree.
pub struct JellyfishTreePruner;

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

        // Clean up leftover directories
        if rocksdb_new.exists() {
            std::fs::remove_dir_all(&rocksdb_new)?;
        }
        if rocksdb_old.exists() {
            std::fs::remove_dir_all(&rocksdb_old)?;
        }

        // Create destination storage
        tracing::info!("creating fresh database at {:?}", rocksdb_new);
        let new_storage = Storage::load(rocksdb_new.clone(), SUBSTORE_PREFIXES.clone()).await?;
        let new_db = new_storage.db();

        // Configure pruning
        let chunk_size: usize = std::env::var("PRUNE_CHUNK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100_000);

        let prune_config = PruneConfig {
            chunk_size,
            ..Default::default()
        };

        // Prune main store using cnidarium
        tracing::info!(chunk_size, "pruning main store");
        let report = prune_main_substore(
            &storage,
            snapshot,
            &new_storage,
            version,
            &prune_config,
        )?;

        tracing::info!(
            keys_processed = report.keys_processed,
            nodes_before = report.nodes_before,
            nodes_after = report.nodes_after,
            "main store pruned (root hash verified via range proofs)"
        );

        /* **************** copy auxiliary and substore column families **************** */
        tracing::info!("copying auxiliary column families from old database");
        let main_aux_cfs = [
            "config",
            "substore--jmt-keys",
            "substore--jmt-keys-by-keyhash",
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

        // Close databases before swap
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

        // Clean up LOG.old files
        for entry in std::fs::read_dir(&rocksdb_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("LOG.old") {
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
