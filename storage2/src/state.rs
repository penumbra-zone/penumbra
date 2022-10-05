use std::{collections::HashMap, sync::Arc};

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
    snapshot: Arc<Snapshot<'static>>,
    unwritten_changes: HashMap<String, Vec<u8>>,
}

impl State {
    pub(crate) fn new(snapshot: Arc<Snapshot<'static>>) -> Self {
        Self {
            snapshot,
            unwritten_changes: HashMap::new(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction {}
    }
}

#[async_trait]
impl StateRead for State {
    fn get_raw(&self, key: String) -> Option<Vec<u8>> {
        // If the key is available in the unwritten_changes cache, return it.
        if let Some(value) = self.unwritten_changes.get(&key) {
            return Some(value.clone());
        }

        // If the key is available in the snapshot, return it.
        self.snapshot.get_raw(key)
    }
}
