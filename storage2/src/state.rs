use futures::future::BoxFuture;
use std::{collections::BTreeMap, sync::Arc};
use tracing::Span;

use anyhow::Result;
use async_trait::async_trait;

mod read;
mod transaction;
mod write;
use jmt::storage::{NodeBatch, TreeWriter};
pub use read::StateRead;
pub use transaction::Transaction as StateTransaction;
pub use write::StateWrite;

use crate::snapshot::Snapshot;

/// State is a lightweight copy-on-write fork of the chain state,
/// implemented as a RYW cache over a pinned JMT version.
pub struct State {
    snapshot: Arc<Snapshot>,
    // A `None` value represents deletion.
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    // A `None` value represents deletion.
    pub(crate) sidecar_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
}

impl State {
    pub(crate) fn new(snapshot: Arc<Snapshot>) -> Self {
        Self {
            snapshot,
            unwritten_changes: BTreeMap::new(),
            sidecar_changes: BTreeMap::new(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
    }
}

#[async_trait]
impl StateRead for State {
    fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // If the key is available in the unwritten_changes cache, return it.
        if let Some(v) = self.unwritten_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the snapshot, return it.
        self.snapshot.get_raw(key)
    }

    fn get_sidecar_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // If the key is available in the sidecar cache, return it.
        if let Some(v) = self.sidecar_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the snapshot, return it.
        self.snapshot.get_sidecar(key)
    }
}
