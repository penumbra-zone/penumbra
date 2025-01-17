use std::sync::Arc;

// HashMap is okay here because we don't care about ordering of substore roots.
use std::collections::HashMap;

use crate::{
    cache::Cache,
    store::{multistore, substore::SubstoreConfig},
    RootHash,
};

/// A staged write batch that can be committed to RocksDB.
///
/// This allows for write batches to be prepared and committed at a later time.
pub struct StagedWriteBatch {
    /// The write batch to commit to RocksDB.
    pub(crate) write_batch: rocksdb::WriteBatch,
    /// The new version of the chain state.
    pub(crate) version: jmt::Version,
    /// The new versions of each substore.
    pub(crate) multistore_versions: multistore::MultistoreCache,
    /// The root hash of the chain state corresponding to this set of changes.
    pub(crate) root_hash: RootHash,
    /// The configs, root hashes, and new versions of each substore
    /// that was updated in this batch.
    #[allow(clippy::disallowed_types)]
    pub(crate) substore_roots: HashMap<Arc<SubstoreConfig>, (RootHash, u64)>,
    /// Whether or not to perform a migration.
    pub(crate) perform_migration: bool,
    /// A lightweight copy of the changeset, this is useful to provide
    /// a stream of changes to subscribers.
    pub(crate) changes: Arc<Cache>,
}

impl StagedWriteBatch {
    /// Returns the new version of the chain state corresponding to this set of changes.
    pub fn version(&self) -> jmt::Version {
        self.version
    }

    /// Returns the root hash of the jmt corresponding to this set of changes.
    pub fn root_hash(&self) -> &RootHash {
        &self.root_hash
    }

    /// Returns the version of a substore in this batch, if it exists
    /// and `None` otherwise.
    pub fn substore_version(&self, prefix: &str) -> Option<jmt::Version> {
        let Some(substore_config) = self
            .multistore_versions
            .config
            .find_substore(prefix.as_bytes())
        else {
            return None;
        };

        self.multistore_versions.get_version(&substore_config)
    }
}
