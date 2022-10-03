use std::collections::HashMap;

use jmt::{
    storage::{TreeReader, TreeWriter},
    JellyfishMerkleTree,
};

mod transaction;
use transaction::Transaction as StateTransaction;

/// State is a lightweight copy-on-write fork of the chain state,
/// implemented as a RYW cache over a pinned JMT version.
pub(crate) struct State<'a, R: TreeReader + TreeWriter> {
    cache: HashMap<jmt::KeyHash, jmt::OwnedValue>,
    jmt_version: jmt::Version,
    jmt: &'a JellyfishMerkleTree<'a, R>,
}

impl<'a, R: TreeReader + TreeWriter> State<'a, R> {
    pub fn new() -> Self {
        Self {
            cache: todo!(),
            jmt_version: todo!(),
            jmt: todo!(),
        }
    }

    pub fn begin_transaction(&mut self) -> StateTransaction<'a, R> {
        StateTransaction::new()
    }
}

impl<'a, R: TreeReader + TreeWriter> StateRead for State<'a, R> {
    fn get(&self, key: jmt::KeyHash) -> Option<&jmt::OwnedValue> {
        todo!()
    }
}

pub trait StateRead {
    /// Get
    fn get(&self, key: jmt::KeyHash) -> Option<&jmt::OwnedValue>;
}

pub trait StateWrite {
    /// Copy-on-write put
    fn put(&mut self, key: jmt::KeyHash, value: Option<jmt::OwnedValue>) -> Self;
}
