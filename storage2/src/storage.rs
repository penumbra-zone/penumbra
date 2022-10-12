use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use parking_lot::RwLock;
use rocksdb::DB;

use crate::snapshot::Snapshot;
use crate::State;

pub struct Storage {
    latest_snapshot: RwLock<Arc<Snapshot>>,
    db: Arc<DB>,
}

impl Storage {
    pub async fn load(path: PathBuf) -> Result<Self> {
        todo!()
    }

    /// Returns the latest version (block height) of the tree recorded by the
    /// `Storage`, or `None` if the tree is empty.
    pub async fn latest_version(&self) -> Result<Option<jmt::Version>> {
        todo!()
    }

    /// Returns a new [`State`] on top of the latest version of the tree.
    pub async fn state(&self) -> State {
        State::new(self.latest_snapshot.read().clone())
    }

    // TODO: this probably can't be 'static long-term
    pub async fn apply(&'static mut self, state: State) {
        // TODO: 1. write the index tables and JMT to RocksDB
        // 2. update the snapshot
        // TODO: set jmt_version correctly
        let jmt_version = 0;
        let snapshot = self.db.snapshot();
        // Obtain the write-lock for the latest snapshot, and replace it with the new snapshot.
        let mut guard = self.latest_snapshot.write();
        *guard = Arc::new(Snapshot::new(snapshot, jmt_version));
        // Drop the write-lock (this will happen implicitly anyways, but it's good to be explicit).
        drop(guard);
    }
}
