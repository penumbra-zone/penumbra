use std::{any::Any, collections::BTreeMap, future::Future, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use futures::{FutureExt, Stream};
use tendermint::abci;

use crate::State;

use super::{
    read::{nonconsensus_prefix_raw_with_cache, prefix_keys_with_cache, prefix_raw_with_cache},
    StateRead, StateWrite,
};

/// A set of pending changes to a [`State`] instance, supporting both writes and reads.
pub struct Transaction<'a> {
    /// The `State` instance this transaction will modify.
    ///
    /// Holding on to a &mut reference ensures there can only be one live transaction at a time.
    state: &'a mut State,
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonconsensus storage).
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Unwritten changes to the object store.  A `None` value means a deletion.
    pub(crate) object_changes: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    /// A list of ABCI events that occurred while building this set of state changes.
    pub(crate) events: Vec<abci::Event>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: BTreeMap::new(),
            nonconsensus_changes: BTreeMap::new(),
            object_changes: BTreeMap::new(),
            events: Vec::new(),
        }
    }

    /// Applies this transaction's writes to its parent [`State`], completing the transaction.
    ///
    /// Returns a list of all the events that occurred while building the transaction.
    pub fn apply(self) -> Vec<abci::Event> {
        // Write the unwritten consensus-critical changes to the state:
        self.state.unwritten_changes.extend(self.unwritten_changes);

        // Write the unwritten nonconsensus changes to the state:
        self.state
            .nonconsensus_changes
            .extend(self.nonconsensus_changes);

        for (k, v_or_deletion) in self.object_changes {
            match v_or_deletion {
                Some(v) => {
                    self.state.ephemeral_objects.insert(k, v);
                }
                None => {
                    self.state.ephemeral_objects.remove(&k);
                }
            }
        }

        self.events
    }

    /// Aborts this transaction, discarding its writes.
    ///
    /// Convienence method for [`std::mem::drop`].
    pub fn abort(self) {}
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.unwritten_changes.insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.unwritten_changes.insert(key, None);
    }

    fn nonconsensus_delete(&mut self, key: Vec<u8>) {
        self.nonconsensus_changes.insert(key, None);
    }

    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nonconsensus_changes.insert(key, Some(value));
    }

    fn object_put<T: Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        self.object_changes.insert(key, Some(Box::new(value)));
    }

    fn object_delete(&mut self, key: &'static str) {
        self.object_changes.insert(key, None);
    }

    fn record(&mut self, event: abci::Event) {
        self.events.push(event)
    }
}

#[async_trait]
impl<'tx> StateRead for Transaction<'tx> {
    fn get_raw(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'static>> {
        // We want to return a 'static future, so we need to get all our references
        // to &self done upfront, before we bundle the results into a future.

        // If the key is available in the unwritten_changes cache, extract it now,
        // so we can move it into the future we'll return.
        let cached_value = self.unwritten_changes.get(key).cloned();
        // Prepare a query to the state; this won't start executing until we poll it.
        let state_value = self.state.get_raw(key);

        async move {
            match cached_value {
                // If the key is available in the unwritten_changes cache, return it.
                Some(v) => Ok(v),
                // Otherwise, if the key is available in the state, return it.
                None => state_value.await,
            }
        }
        .boxed()
    }

    async fn nonconsensus_get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // If the key is available in the nonconsensus cache, return it.
        if let Some(v) = self.nonconsensus_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the state, return it.
        self.state.nonconsensus_get_raw(key).await
    }

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Sync + Send + 'a>> {
        prefix_raw_with_cache(self.state, &self.unwritten_changes, prefix)
    }

    fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send + 'a>> {
        prefix_keys_with_cache(self.state, &self.unwritten_changes, prefix)
    }

    fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Sync + Send + 'a>> {
        nonconsensus_prefix_raw_with_cache(self.state, &self.nonconsensus_changes, prefix)
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        if let Some(v_or_deletion) = self.object_changes.get(key) {
            return v_or_deletion.as_ref().and_then(|v| v.downcast_ref());
        }
        self.state.object_get(key)
    }
}
