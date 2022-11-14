use crate::{Message, Protobuf, StateReadProto};

use std::fmt::Debug;

use penumbra_storage::StateWrite;

pub trait StateWriteProto: StateWrite + StateReadProto + Send + Sync {
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
}
