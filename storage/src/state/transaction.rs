use std::{any::Any, pin::Pin};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use tendermint::abci;

use crate::{
    future::{CacheFuture, SnapshotFuture},
    State,
};

use super::{Cache, StateRead, StateWrite};

/// A set of pending changes to a [`State`] instance, supporting both writes and reads.
pub struct Transaction<'a> {
    /// The `State` instance this transaction will modify.
    ///
    /// Holding on to a &mut reference ensures there can only be one live transaction at a time.
    state: &'a mut State,
    cache: Cache,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(state: &'a mut State) -> Self {
        Self {
            state,
            cache: Default::default(),
        }
    }

    /// Applies this transaction's writes to its parent [`State`], completing the transaction.
    ///
    /// Returns a list of all the events that occurred while building the transaction.
    pub fn apply(mut self) -> Vec<abci::Event> {
        let events = std::mem::take(&mut self.cache.events);

        self.state.cache.merge(self.cache);

        events
    }

    /// Aborts this transaction, discarding its writes.
    ///
    /// Convienence method for [`std::mem::drop`].
    pub fn abort(self) {}
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.cache.unwritten_changes.insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.cache.unwritten_changes.insert(key, None);
    }

    fn nonconsensus_delete(&mut self, key: Vec<u8>) {
        self.cache.nonconsensus_changes.insert(key, None);
    }

    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.cache.nonconsensus_changes.insert(key, Some(value));
    }

    fn object_put<T: Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        self.cache
            .ephemeral_objects
            .insert(key, Some(Box::new(value)));
    }

    fn object_delete(&mut self, key: &'static str) {
        self.cache.ephemeral_objects.insert(key, None);
    }

    fn record(&mut self, event: abci::Event) {
        self.cache.events.push(event)
    }
}

#[async_trait]
impl<'tx> StateRead for Transaction<'tx> {
    type GetRawFut = CacheFuture<CacheFuture<SnapshotFuture>>;
    type PrefixRawStream<'a> = Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>>
    where
        Self: 'a;
    type PrefixKeysStream<'a> = Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>>
    where
        Self: 'a;
    type NonconsensusPrefixRawStream<'a> = Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>>
    where
        Self: 'a;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        self.cache.get_raw_or_else(key, || self.state.get_raw(key))
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        self.cache
            .nonconsensus_get_raw_or_else(key, || self.state.nonconsensus_get_raw(key))
    }

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Send + 'a>> {
        self.cache.prefix_raw(prefix, self.state.prefix_raw(prefix))
    }

    fn prefix_keys<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>> {
        self.cache
            .prefix_keys(prefix, self.state.prefix_keys(prefix))
    }

    fn nonconsensus_prefix_raw<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<u8>, Vec<u8>)>> + Send + 'a>> {
        self.cache
            .nonconsensus_prefix_raw(prefix, self.state.nonconsensus_prefix_raw(prefix))
    }

    fn object_get<T: Any + Send + Sync>(&self, key: &'static str) -> Option<&T> {
        if let Some(v_or_deletion) = self.cache.ephemeral_objects.get(key) {
            return v_or_deletion.as_ref().and_then(|v| v.downcast_ref());
        }
        self.state.object_get(key)
    }
}
