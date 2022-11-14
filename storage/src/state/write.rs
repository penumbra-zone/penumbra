use std::{any::Any, fmt::Debug};
use tendermint::abci;

use penumbra_proto::{Message, Protobuf};

use crate::StateRead;

/// Write access to chain state.
pub trait StateWrite: StateRead + Send + Sync {
    /// Puts raw bytes into the verifiable key-value store with the given key.
    fn put_raw(&mut self, key: String, value: Vec<u8>);

    /// Puts a domain type into the verifiable key-value store with the given key.
    fn put<D, P>(&mut self, key: String, value: D)
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        self.put_proto(key, P::from(value));
    }

    /// Puts a proto type into the verifiable key-value store with the given key.
    fn put_proto<P>(&mut self, key: String, value: P)
    where
        P: Message + Default + Debug,
    {
        self.put_raw(key, value.encode_to_vec());
    }

    /// Delete a key from the verifiable key-value store.
    fn delete(&mut self, key: String);

    /// Puts raw bytes into the non-verifiable key-value store with the given key.
    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>);

    /// Delete a key from non-verifiable key-value storage.
    fn nonconsensus_delete(&mut self, key: Vec<u8>);

    /// Puts an object into the ephemeral object store with the given key.
    fn object_put<T: Any + Send + Sync>(&mut self, key: &'static str, value: T);

    /// Deletes a key from the ephemeral object store.
    fn object_delete(&mut self, key: &'static str);

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

    fn object_put<T: Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        (**self).object_put(key, value)
    }

    fn object_delete(&mut self, key: &'static str) {
        (**self).object_delete(key)
    }

    fn record(&mut self, event: abci::Event) {
        (**self).record(event)
    }
}
