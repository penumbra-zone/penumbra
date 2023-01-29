use crate::{DomainType, Message};

use std::fmt::Debug;

use penumbra_storage::StateWrite;

pub trait StateWriteProto: StateWrite + Send + Sync {
    /// Puts a domain type into the verifiable key-value store with the given key.
    fn put<D>(&mut self, key: String, value: D)
    where
        D: DomainType,
        <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
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
}
impl<T: StateWrite + ?Sized> StateWriteProto for T {}
