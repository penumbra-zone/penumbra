use crate::State;

use super::StateWrite;

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction<'a> {
    // TODO determine which fields to include (#1490)
    // cache: HashMap<jmt::KeyHash, jmt::OwnedValue>,
    // unwritten_changes: Vec<(jmt::KeyHash, jmt::OwnedValue)>,
    state: &'a mut State,
}

impl<'a> Transaction<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn commit(self) {
        // Write unwritten_changes to our parent in-memory state fork.
        // The state will not be written to storage until `State::commit` is called.
        // `self` will be consumed afterwards
        todo!()
    }
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        todo!()
    }

    fn delete(&mut self, key: String) {
        todo!()
    }
}
