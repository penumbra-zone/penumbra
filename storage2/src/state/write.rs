use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;

use penumbra_proto::{Message, Protobuf};

#[async_trait]
pub trait StateWrite {
    /// Copy-on-write put
    fn put_raw(&mut self, key: String, value: Vec<u8>) -> Result<()>;

    /// Sets a domain type from the State.
    fn put<D, P>(&self, key: String, value: D) -> Result<()>
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

    /// Puts a proto type on the State.
    fn put_proto<D, P>(&self, key: String, value: P) -> Result<()>
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

    /// Delete a key from state.
    fn delete(&mut self, key: String) -> Result<()>;

    /// Put a key/value pair into non-consensus-critical ("sidecar") state.
    fn put_sidecar(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}
