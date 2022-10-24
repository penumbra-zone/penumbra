use std::{collections::BTreeMap, pin::Pin, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

mod read;
mod transaction;
mod write;
use futures::Stream;
pub use read::StateRead;
use tokio::sync::mpsc;
use tracing::Span;
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

    async fn prefix_raw(
        &self,
        prefix: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = (String, Box<[u8]>)> + Send + '_>>> {
        // TODO: Interleave the unwritten_changes cache with the snapshot.
        todo!()
        // let span = Span::current();
        // let db = self.0.db;
        // let rocksdb_snapshot = self.0.rocksdb_snapshot.clone();
        // let mut options = rocksdb::ReadOptions::default();
        // options.set_iterate_range(rocksdb::PrefixRange(prefix.as_bytes()));
        // let mode = rocksdb::IteratorMode::Start;

        // let (tx, rx) = mpsc::channel(100);

        // tokio::task::Builder::new()
        //     .name("Snapshot::prefix_raw")
        //     .spawn_blocking(move || {
        //         span.in_scope(|| {
        //             let jmt_cf = db.cf_handle("jmt").expect("jmt column family not found");
        //             let iter = rocksdb_snapshot.iterator_cf_opt(jmt_cf, options, mode);
        //             for i in iter {
        //                 tx.blocking_send(i?)?;
        //             }
        //             Ok::<(), anyhow::Error>(())
        //         })
        //     })?
        //     .await??;

        // Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
