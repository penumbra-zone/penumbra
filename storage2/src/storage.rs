use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use rocksdb::DB;

use crate::snapshot::Snapshot;
use crate::State;

#[derive(Clone)]
pub struct Storage {
    snapshot: Arc<Snapshot>,
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
    pub async fn state(&self) -> Result<State> {
        Ok(State::new(self.snapshot.clone()))
    }

    /// Like [`Self::state`], but bundles in a [`tonic`] error conversion.
    ///
    /// This is useful for implementing gRPC services that query the storage:
    /// each gRPC request can create an ephemeral [`State`] pinning the current
    /// version at the time the request was received, and then query it using
    /// component `View`s to handle the request.
    pub async fn state_tonic(&self) -> std::result::Result<State, tonic::Status> {
        self.state()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }

    // TODO: this probably can't be 'static long-term
    pub async fn apply(&'static mut self, state: State) {
        // TODO: 1. write the index tables and JMT to RocksDB
        // 2. update the snapshot
        // TODO: set jmt_version correctly
        let jmt_version = 0;
        let snapshot = self.db.snapshot();
        self.snapshot = Arc::new(Snapshot::new(snapshot, jmt_version));
    }
}
