use std::fmt::Debug;

use async_trait::async_trait;

use penumbra_proto::{Message, Protobuf};

#[async_trait]
pub trait StateRead {
    /// Get
    fn get_raw(&self, key: String) -> Option<Vec<u8>>;

    /// Gets a domain type from the State.
    fn get<D, P>(&self, key: String) -> Option<D>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        todo!()
    }

    /// Gets a proto type from the State.
    fn get_proto<D, P>(&self, key: String) -> Option<P>
    where
        D: Protobuf<P>,
        // TODO: does this get less awful if P is an associated type of D?
        P: Message + Default,
        P: From<D>,
        D: TryFrom<P> + Clone + Debug,
        <D as TryFrom<P>>::Error: Into<anyhow::Error>,
    {
        todo!()
    }
}
