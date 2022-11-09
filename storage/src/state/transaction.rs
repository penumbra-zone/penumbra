use std::{any::Any, collections::BTreeMap, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use tendermint::abci;

use crate::State;

use super::{read::prefix_raw_with_cache, StateRead, StateWrite};

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
    pub(crate) object_changes: BTreeMap<String, Option<Box<dyn Any + Send + Sync>>>,
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

    fn delete_nonconsensus(&mut self, key: Vec<u8>) {
        self.nonconsensus_changes.insert(key, None);
    }

    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nonconsensus_changes.insert(key, Some(value));
    }

    fn put_ephemeral<T: Any + Send + Sync>(&mut self, key: String, value: T) {
        self.object_changes.insert(key, Some(Box::new(value)));
    }

    fn delete_ephemeral(&mut self, key: String) {
        self.object_changes.insert(key, None);
    }

    fn record(&mut self, event: abci::Event) {
        self.events.push(event)
    }
}

#[async_trait]
impl<'tx> StateRead for Transaction<'tx> {
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

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Sync + Send + 'a>> {
        prefix_raw_with_cache(self.state, &self.unwritten_changes, prefix)
    }

    fn get_ephemeral<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        if let Some(v_or_deletion) = self.object_changes.get(key) {
            return v_or_deletion.as_ref().and_then(|v| v.downcast_ref());
        }
        self.state.get_ephemeral(key)
    }
}
