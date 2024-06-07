use tendermint::abci::Event;

#[derive(Debug)]
pub struct ContextualizedEvent {
    pub event: Event,
    pub block_height: u64,
    pub tx_hash: Option<[u8; 32]>,
    /// The rowid of the event in the local database.
    ///
    /// Note that this is a purely local identifier and won't be the same across
    /// different event databases.
    pub local_rowid: i64,
}

impl AsRef<Event> for ContextualizedEvent {
    fn as_ref(&self) -> &Event {
        &self.event
    }
}
