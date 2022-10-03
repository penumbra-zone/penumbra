use std::collections::HashMap;

use anyhow::Result;

use jmt::storage::{TreeReader, TreeWriter};

use super::{State, StateWrite};

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub(crate) struct Transaction<'a, R: TreeReader + TreeWriter> {
    // TODO: should higher-level types be used here instead of jmt types?
    cache: HashMap<jmt::KeyHash, jmt::OwnedValue>,
    unwritten_changes: Vec<(jmt::KeyHash, jmt::OwnedValue)>,
    state: &'a State<'a, R>,
}

impl<'a, R: TreeReader + TreeWriter> Transaction<'a, R> {
    pub fn new() -> Self {
        Self {
            cache: todo!(),
            unwritten_changes: todo!(),
            state: todo!(),
        }
    }

    pub fn commit(self) -> Result<()> {
        // Write unwritten_changes to the state
        // `self` will be consumed afterwards
        todo!()
    }
}

impl<'a, R: TreeReader + TreeWriter> StateWrite for Transaction<'a, R> {
    fn put(&mut self, key: jmt::KeyHash, value: Option<jmt::OwnedValue>) -> Transaction<'a, R> {
        todo!()
    }
}

pub trait StateRead {
    /// Get
    fn get(&self, key: jmt::KeyHash) -> Option<&jmt::OwnedValue>;
}
