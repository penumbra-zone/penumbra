use super::StateWrite;

/// Represents a transactional set of changes to a `State` fork,
/// implemented as a RYW cache over a `State`.
pub struct Transaction {
    // TODO determine which fields to include (#1490)
    // cache: HashMap<jmt::KeyHash, jmt::OwnedValue>,
    // unwritten_changes: Vec<(jmt::KeyHash, jmt::OwnedValue)>,
    // state: &'a mut State<'a>,
}

impl Transaction {
    pub fn commit(self) {
        // Write unwritten_changes to our parent in-memory state fork.
        // The state will not be written to storage until `State::commit` is called.
        // `self` will be consumed afterwards
        todo!()
    }
}

impl StateWrite for Transaction {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        todo!()
    }

    fn delete(&mut self, key: String) {
        todo!()
    }
}
