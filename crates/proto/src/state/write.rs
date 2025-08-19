use crate::{event::ProtoEvent, DomainType, Message};

use std::fmt::Debug;

use cnidarium::StateWrite;

pub trait StateWriteProto: StateWrite + Send + Sync {
    /// Puts a domain type into the verifiable key-value store with the given key.
    fn put<D>(&mut self, key: String, value: D)
    where
        D: DomainType,
        anyhow::Error: From<<D as TryFrom<D::Proto>>::Error>,
    {
        self.put_proto(key, D::Proto::from(value));
    }

    /// Puts a proto type into the verifiable key-value store with the given key.
    fn put_proto<P>(&mut self, key: String, value: P)
    where
        P: Message + Default + Debug,
    {
        self.put_raw(key, value.encode_to_vec());
    }

    /// Puts a domain type into the nonverifiable key-value store with the given key
    fn nonverifiable_put<D>(&mut self, key: Vec<u8>, value: D)
    where
        D: DomainType,
        anyhow::Error: From<<D as TryFrom<D::Proto>>::Error>,
    {
        self.nonverifiable_put_raw(key, value.encode_to_vec());
    }

    /// Puts a proto type into the verifiable key-value store with the given key.
    fn nonverifiable_put_proto<P>(&mut self, key: Vec<u8>, value: P)
    where
        P: Message + Default + Debug,
    {
        self.nonverifiable_put_raw(key, value.encode_to_vec());
    }

    /// Records a Protobuf message as a typed ABCI event.
    fn record_proto<E>(&mut self, proto_event: E)
    where
        E: ProtoEvent,
    {
        self.record(proto_event.into_event())
    }
}
impl<T: StateWrite + ?Sized> StateWriteProto for T {}
