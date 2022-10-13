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
    unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
}

impl State {
    pub(crate) fn new(snapshot: Arc<Snapshot>) -> Self {
        Self {
            snapshot,
            unwritten_changes: BTreeMap::new(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
    }

    // Apply the unwritten changes of a transaction to this state fork.
    pub fn apply_transaction(&mut self, transaction: StateTransaction) {
        for (key, value) in transaction.unwritten_changes.iter() {
            self.unwritten_changes.insert(key.clone(), value.clone());
        }
    }
}

#[async_trait]
impl StateRead for State {
    fn get_raw(&self, key: String) -> Option<Vec<u8>> {
        // If the key is available in the unwritten_changes cache, return it.
        // A `None` value represents that the key has been deleted.
        if let Some(value) = self.unwritten_changes.get(&key) {
            return value.clone();
        }

        // If the key is available in the snapshot, return it.
        self.snapshot.get_raw(key)
    }
}
