use std::{any::Any, collections::BTreeMap};

use tendermint::abci;

/// A cache of changes to the state of the blockchain.
///
/// Used internally by `State` and `StateTransaction`.
#[derive(Default, Debug)]
pub struct Cache {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonconsensus storage).
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Unwritten changes to the object store.  A `None` value means a deletion.
    pub(crate) ephemeral_objects: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    /// A list of ABCI events that occurred while building this set of state changes.
    pub(crate) events: Vec<abci::Event>,
}

impl Cache {
    /// Merge the given cache with this one, taking its writes in place of ours.
    pub fn merge(&mut self, other: Cache) {
        self.unwritten_changes.extend(other.unwritten_changes);
        self.nonconsensus_changes.extend(other.nonconsensus_changes);
        self.ephemeral_objects.extend(other.ephemeral_objects);
        self.events.extend(other.events);
    }

    /// Returns `true` if there are cached writes on top of the snapshot, and `false` otherwise.
    pub fn is_dirty(&self) -> bool {
        !(self.unwritten_changes.is_empty()
            && self.nonconsensus_changes.is_empty()
            && self.ephemeral_objects.is_empty())
    }
}
