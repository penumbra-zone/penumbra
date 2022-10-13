use crate::State;

use super::StateWrite;

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction<'a> {
    pub(crate) unwritten_changes: Vec<(String, Option<Vec<u8>>)>,
    state: &'a mut State,
}

impl<'a> Transaction<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: Vec::new(),
        }
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
