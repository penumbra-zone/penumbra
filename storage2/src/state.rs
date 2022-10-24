use std::{collections::BTreeMap, pin::Pin, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

mod read;
mod transaction;
mod write;
use futures::Stream;
pub use read::StateRead;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
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
        // Interleave the unwritten_changes cache with the snapshot.
        let (tx, rx) = mpsc::channel(100);

        let mut snapshotted_stream = self.snapshot.prefix_raw(prefix).await?;
        let mut snapshotted_match = snapshotted_stream.next().await;
        for (key, value) in self.unwritten_changes.iter() {
            // Iterate the unwritten_changes cache (sorted by key) until we reach the keys
            // that match the prefix.
            //
            // If value is `None`, then the key has been deleted, and we should skip it.
            if !key.starts_with(prefix) || value.is_none() {
                continue;
            }

            let value = value.clone().unwrap();

            // This key matches the prefix.
            // While the snapshot prefix stream returns keys that lexicographically precede this key,
            // return those.
            while let Some((snapshotted_key, snapshotted_value)) = snapshotted_match {
                if &snapshotted_key < key {
                    // The snapshot key is less than the unwritten_changes key, so return
                    // the snapshot key.
                    tx.send((snapshotted_key, snapshotted_value)).await?;
                    // And then advance the snapshot stream to the next match.
                    snapshotted_match = snapshotted_stream.next().await;
                } else {
                    // Keep this match around for another iteration.
                    snapshotted_match = Some((snapshotted_key, snapshotted_value));
                }
            }

            // All snapshot matches preceding this unwritten_changes key have been sent to the channel,
            // so send this key.
            tx.send((key.to_string(), value.into_boxed_slice())).await?;
        }

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
