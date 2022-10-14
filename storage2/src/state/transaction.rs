use crate::State;

use anyhow::Result;

use super::StateWrite;

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction<'a> {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: Vec<(String, Option<Vec<u8>>)>,
    /// Unwritten changes to non-consensus-critical state (stored in the sidecar).
    pub(crate) sidecar_changes: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    state: &'a mut State,
}

impl<'a> Transaction<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: Vec::new(),
            sidecar_changes: Vec::new(),
        }
    }
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) -> Result<()> {
        self.unwritten_changes.push((key, Some(value)));
    }

    fn put_sidecar(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        todo!()
    }

    fn delete(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
