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
pub use transaction::Transaction as StateTransaction;
pub use write::StateWrite;

use crate::snapshot::Snapshot;

/// State is a lightweight copy-on-write fork of the chain state,
/// implemented as a RYW cache over a pinned JMT version.
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

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction::new(self)
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

        // Range the unwritten_changes cache (sorted by key) starting with the keys matching the prefix,
        // until we reach the keys that no longer match the prefix.
        let unwritten_changes_iter = self
            .unwritten_changes
            .range(prefix.to_string()..)
            .take_while(|(k, _)| (**k).starts_with(prefix));

        // Maybe it would be possible to simplify this by using `async-stream` and implementing something similar to `itertools::merge_by`.

        for (key, value) in unwritten_changes_iter {
            // If value is `None`, then the key has been deleted, and we should skip it.
            if value.is_none() {
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
                    break;
                }
            }

            // All snapshot matches preceding this unwritten_changes key have been sent to the channel,
            // so send this key.
            tx.send((key.to_string(), value.into_boxed_slice())).await?;
        }

        // Send any remaining data from the snapshot stream.
        while let Some((snapshotted_key, snapshotted_value)) = snapshotted_match {
            tx.send((snapshotted_key, snapshotted_value)).await?;
            // Advance the snapshot stream to the next match.
            snapshotted_match = snapshotted_stream.next().await;
        }

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
