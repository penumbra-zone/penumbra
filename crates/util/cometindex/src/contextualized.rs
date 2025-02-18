use tendermint::abci::Event;

#[derive(Clone, Copy, Debug)]
pub struct ContextualizedEvent<'block> {
    pub event: &'block Event,
    pub block_height: u64,
    pub tx: Option<([u8; 32], &'block [u8])>,
    /// The rowid of the event in the local database.
    ///
    /// Note that this is a purely local identifier and won't be the same across
    /// different event databases.
    pub local_rowid: i64,
}

impl<'block> ContextualizedEvent<'block> {
    pub fn tx_hash(&self) -> Option<[u8; 32]> {
        self.tx.map(|x| x.0)
    }

    pub fn tx_data(&self) -> Option<&'block [u8]> {
        self.tx.map(|x| x.1)
    }
}

impl<'tx> AsRef<Event> for ContextualizedEvent<'tx> {
    fn as_ref(&self) -> &Event {
        &self.event
    }
}
