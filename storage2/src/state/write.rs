use std::fmt::Debug;

use penumbra_proto::{Message, Protobuf};

/// Write access to chain state.
pub trait StateWrite {
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
    fn put_proto<D, P>(&mut self, key: String, value: P)
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        self.put_raw(key, value.encode_to_vec());
    }

    /// Delete a key from the verifiable key-value store.
    fn delete(&mut self, key: String);

    /// Delete a key from non-verifiable key-value storage.
    fn delete_nonconsensus(&mut self, key: Vec<u8>);

    /// Puts raw bytes into the non-verifiable key-value store with the given key.
    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>);
}
