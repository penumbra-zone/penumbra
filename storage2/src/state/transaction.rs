use std::collections::BTreeMap;

use crate::State;

use anyhow::Result;

use super::StateWrite;

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction<'a> {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the sidecar).
    pub(crate) sidecar_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    state: &'a mut State,
}

impl<'a> Transaction<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: BTreeMap::new(),
            sidecar_changes: BTreeMap::new(),
        }
    }
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.unwritten_changes.insert(key, Some(value));
    }

    fn put_sidecar(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.sidecar_changes.insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.unwritten_changes.insert(key, None);
    }

    fn delete_sidecar(&mut self, key: Vec<u8>) {
        self.sidecar_changes.insert(key, None);
    }
}
