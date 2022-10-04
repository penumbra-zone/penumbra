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
    pub fn finish(self) {
        // Write unwritten_changes to the state
        // `self` will be consumed afterwards
        todo!()
    }
}

impl StateWrite for Transaction {
    fn put(&mut self, key: String, value: jmt::OwnedValue) {
        todo!()
    }

    fn delete(&mut self, key: String) {
        todo!()
    }
}

pub trait StateRead {
    /// Get
    fn get(&self, key: String) -> Option<&jmt::OwnedValue>;
}
