use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::{collections::BTreeMap, pin::Pin};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::State;

use super::{StateRead, StateWrite};

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction<'a> {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonconsensus storage).
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    state: &'a mut State,
    pub(crate) failed: bool,
    pub(crate) failure_reason: String,
}

impl<'a> Transaction<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: BTreeMap::new(),
            nonconsensus_changes: BTreeMap::new(),
            failed: false,
            failure_reason: String::new(),
        }
    }

    pub fn fail(&mut self, reason: String) {
        self.failed = true;
        self.failure_reason = reason;
    }

    pub fn commit(self) -> Result<()> {
        if self.failed {
            return Err(anyhow::anyhow!("transaction failed").context(self.failure_reason));
        }

        // Write the unwritten consensus-critical changes to the state:
        self.state.unwritten_changes.extend(self.unwritten_changes);

        // Write the unwritten nonconsensus changes to the state:
        self.state
            .nonconsensus_changes
            .extend(self.nonconsensus_changes);

        Ok(())
    }
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.unwritten_changes.insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.unwritten_changes.insert(key, None);
    }

    fn delete_nonconsensus(&mut self, key: Vec<u8>) {
        self.nonconsensus_changes.insert(key, None);
    }

    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nonconsensus_changes.insert(key, Some(value));
    }
}

#[async_trait]
impl<'a> StateRead for Transaction<'a> {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // If the key is available in the unwritten_changes cache, return it.
        if let Some(v) = self.unwritten_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the state, return it.
        self.state.get_raw(key).await
    }

    async fn get_nonconsensus(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // If the key is available in the nonconsensus cache, return it.
        if let Some(v) = self.nonconsensus_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the state, return it.
        self.state.get_nonconsensus(key).await
    }

    async fn prefix_raw(
        &self,
        prefix: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = (String, Box<[u8]>)> + Send + '_>>> {
        // Interleave the unwritten_changes cache with the state.
        let (tx, rx) = mpsc::channel(100);

        let mut state_stream = self.state.prefix_raw(prefix).await?;
        let mut state_match = state_stream.next().await;

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
            // While the state prefix stream returns keys that lexicographically precede this key,
            // return those.
            while let Some((state_key, state_value)) = state_match {
                if &state_key < key {
                    // The state key is less than the unwritten_changes key, so return
                    // the state key.
                    tx.send((state_key, state_value)).await?;
                    // And then advance the state stream to the next match.
                    state_match = state_stream.next().await;
                } else {
                    // Keep this match around for another iteration.
                    state_match = Some((state_key, state_value));
                    break;
                }
            }

            // All state matches preceding this unwritten_changes key have been sent to the channel,
            // so send this key.
            tx.send((key.to_string(), value.into_boxed_slice())).await?;
        }

        // Send any remaining data from the state stream.
        while let Some((state_key, state_value)) = state_match {
            tx.send((state_key, state_value)).await?;
            // Advance the snapshot stream to the next match.
            state_match = state_stream.next().await;
        }

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
