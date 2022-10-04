use std::fmt::Debug;

use async_trait::async_trait;

use penumbra_proto::{Message, Protobuf};

mod transaction;
pub use transaction::Transaction as StateTransaction;

/// State is a lightweight copy-on-write fork of the chain state,
/// implemented as a RYW cache over a pinned JMT version.
pub struct State {
    // TODO: determine which fields to include
    // cache: HashMap<jmt::KeyHash, jmt::OwnedValue>,
    // jmt_version: jmt::Version,
    // jmt: &'a JellyfishMerkleTree<'a, R>,
}

impl State {
    pub fn new() -> Self {
        Self {
            // cache: todo!(),
            // jmt_version: todo!(),
            // jmt: todo!(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction {
        StateTransaction{
            // cache: todo!(),
            // unwritten_changes: todo!(),
            // state: self,
        }
    }
}

#[async_trait]
impl StateRead for State {
    fn get_raw(&self, key: String) -> Option<Vec<u8>> {
        todo!()
    }
}

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

pub trait StateWrite {
    /// Copy-on-write put
    fn put_raw(&mut self, key: String, value: Vec<u8>);

    /// Sets a domain type from the State.
    fn put<D, P>(&self, key: String, value: D)
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
    fn put_proto<D, P>(&self, key: String, value: P)
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
    fn delete(&mut self, key: String);
}
