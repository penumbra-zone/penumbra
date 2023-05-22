use crate::StateRead;
use std::{any::Any, collections::BTreeMap};
use tendermint::abci;

/// Write access to chain state.
pub trait StateWrite: StateRead + Send + Sync {
    /// Puts raw bytes into the verifiable key-value store with the given key.
    fn put_raw(&mut self, key: String, value: Vec<u8>);

    /// Delete a key from the verifiable key-value store.
    fn delete(&mut self, key: String);

    /// Puts raw bytes into the non-verifiable key-value store with the given key.
    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>);

    /// Delete a key from non-verifiable key-value storage.
    fn nonconsensus_delete(&mut self, key: Vec<u8>);

    /// Puts an object into the ephemeral object store with the given key.
    ///
    /// # Panics
    ///
    /// If the object is already present in the store, but its type is not the same as the type of
    /// `value`.
    fn object_put<T: Clone + Any + Send + Sync>(&mut self, key: &'static str, value: T);

    /// Deletes a key from the ephemeral object store.
    fn object_delete(&mut self, key: &'static str);

    /// Merge a set of object changes into this `StateWrite`.
    ///
    /// Unlike `object_put`, this avoids re-boxing values and messing up the downcasting.
    fn object_merge(&mut self, objects: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>);

    /// Record that an ABCI event occurred while building up this set of state changes.
    fn record(&mut self, event: abci::Event);
}

impl<'a, S: StateWrite + Send + Sync> StateWrite for &'a mut S {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        (**self).put_raw(key, value)
    }

    fn delete(&mut self, key: String) {
        (**self).delete(key)
    }

    fn nonconsensus_delete(&mut self, key: Vec<u8>) {
        (**self).nonconsensus_delete(key)
    }

    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        (**self).nonconsensus_put_raw(key, value)
    }

    fn object_put<T: Clone + Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        (**self).object_put(key, value)
    }

    fn object_delete(&mut self, key: &'static str) {
        (**self).object_delete(key)
    }

    fn object_merge(
        &mut self,
        objects: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    ) {
        (**self).object_merge(objects)
    }

    fn record(&mut self, event: abci::Event) {
        (**self).record(event)
    }
}
