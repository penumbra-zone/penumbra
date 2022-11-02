use std::{any::Any, fmt::Debug};

use penumbra_proto::{Message, Protobuf};

use crate::StateRead;

/// Write access to chain state.
pub trait StateWrite: StateRead {
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
        let key = key.to_lowercase();
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
    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>);

    /// Delete a key from non-verifiable key-value storage.
    fn delete_nonconsensus(&mut self, key: Vec<u8>);

    /// Puts an object into the ephemeral object store with the given key.
    fn put_ephemeral<T: Any + Send + Sync>(&mut self, key: String, value: T);

    /// Deletes a key from the ephemeral object store.
    fn delete_ephemeral(&mut self, key: String);
}

impl<'a, S: StateWrite + Send + Sync> StateWrite for &'a mut S {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        (**self).put_raw(key, value)
    }

    fn delete(&mut self, key: String) {
        (**self).delete(key)
    }

    fn delete_nonconsensus(&mut self, key: Vec<u8>) {
        (**self).delete_nonconsensus(key)
    }

    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>) {
        (**self).put_nonconsensus(key, value)
    }

    fn put_ephemeral<T: Any + Send + Sync>(&mut self, key: String, value: T) {
        (**self).put_ephemeral(key, value)
    }

    fn delete_ephemeral(&mut self, key: String) {
        (**self).delete_ephemeral(key)
    }
}
