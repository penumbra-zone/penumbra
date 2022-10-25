use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

mod read;
mod transaction;
mod write;
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
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
}

impl State {
    pub(crate) fn new(snapshot: Arc<Snapshot>) -> Self {
        Self {
            snapshot,
            unwritten_changes: BTreeMap::new(),
            nonconsensus_changes: BTreeMap::new(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
    }

    pub fn apply_transaction(&mut self, transaction: StateTransaction) -> Result<()> {
        if transaction.failed {
            return Err(anyhow::anyhow!("transaction failed").context(transaction.failure_reason));
        }

        // Write the unwritten consensus-critical changes to the state:
        self.unwritten_changes.extend(transaction.unwritten_changes);

        // Write the unwritten nonconsensus changes to the state:
        self.nonconsensus_changes
            .extend(transaction.nonconsensus_changes);

        Ok(())
    }
}

#[async_trait]
impl StateRead for State {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // If the key is available in the unwritten_changes cache, return it.
        if let Some(v) = self.unwritten_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the snapshot, return it.
        self.snapshot.get_raw(key).await
    }

    async fn get_nonconsensus(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // If the key is available in the nonconsensus cache, return it.
        if let Some(v) = self.nonconsensus_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the snapshot, return it.
        self.snapshot.get_nonconsensus(key).await
    }
}
