use async_trait::async_trait;

mod read;
mod transaction;
mod write;
pub use read::StateRead;
pub use transaction::Transaction as StateTransaction;
pub use write::StateWrite;

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
