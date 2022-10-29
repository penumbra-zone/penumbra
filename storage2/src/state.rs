use std::{collections::BTreeMap, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;

mod read;
mod transaction;
mod write;
use futures::Stream;
pub use read::StateRead;
pub use transaction::Transaction as StateTransaction;
pub use write::StateWrite;

use crate::snapshot::Snapshot;

use self::read::prefix_raw_with_cache;

/// A lightweight snapshot of a particular version of the chain state.
///
/// Each [`State`] instance can also be used as a copy-on-write fork to build up
/// changes before committing them to persistent storage.  The
/// [`StateTransaction`] type collects a group of writes, which can then be
/// applied to the (in-memory) [`State`] fork.  Finally, the changes accumulated
/// in the [`State`] instance can be committed to the persistent [`Storage`](crate::Storage).
pub struct State {
    snapshot: Snapshot,
    // A `None` value represents deletion.
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    // A `None` value represents deletion.
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
}

impl State {
    pub(crate) fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            unwritten_changes: BTreeMap::new(),
            nonconsensus_changes: BTreeMap::new(),
        }
    }

    /// Begins a new batch of writes to be transactionally applied to this
    /// [`State`].
    ///
    /// The resulting [`StateTransaction`] captures a `&mut self` reference, so
    /// each [`State`] only allows one live transaction at a time.
    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
    }

    pub fn version(&self) -> jmt::Version {
        self.snapshot.version()
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

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + Sync + 'a>> {
        prefix_raw_with_cache(self, &self.unwritten_changes, prefix)
    }
}
