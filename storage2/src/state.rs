use std::collections::HashMap;

use jmt::{
    storage::{TreeReader, TreeWriter},
    JellyfishMerkleTree,
};

mod transaction;
use transaction::Transaction as StateTransaction;

/// State is a lightweight copy-on-write fork of the chain state,
/// implemented as a RYW cache over a pinned JMT version.
pub(crate) struct State {
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

impl StateRead for State {
    fn get(&self, key: String) -> Option<&jmt::OwnedValue> {
        todo!()
    }
}

pub trait StateRead {
    /// Get
    fn get(&self, key: String) -> Option<&jmt::OwnedValue>;
}

pub trait StateWrite {
    /// Copy-on-write put
    fn put(&mut self, key: String, value: jmt::OwnedValue);

    /// Delete a key from state.
    fn delete(&mut self, key: String);
}
