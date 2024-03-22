use std::{any::Any, collections::BTreeMap, sync::Arc};

use tendermint::abci;

use crate::{
    store::{multistore::MultistoreConfig, substore::SubstoreConfig},
    StateWrite,
};

/// A cache of changes to the state of the blockchain.
///
/// A [`StateDelta`](crate::StateDelta) is `Cache` above a `StateRead`.
#[derive(Default, Debug)]
pub struct Cache {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonverifiable storage).
    pub(crate) nonverifiable_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Unwritten changes to the object store.  A `None` value means a deletion.
    pub(crate) ephemeral_objects: BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    /// A list of ABCI events that occurred while building this set of state changes.
    pub(crate) events: Vec<abci::Event>,
}

impl Cache {
    /// Inspect the cache of unwritten changes to the verifiable state.
    pub fn unwritten_changes(&self) -> &BTreeMap<String, Option<Vec<u8>>> {
        &self.unwritten_changes
    }

    /// Inspect the cache of unwritten changes to the nonverifiable state.
    pub fn nonverifiable_changes(&self) -> &BTreeMap<Vec<u8>, Option<Vec<u8>>> {
        &self.nonverifiable_changes
    }

    /// Merge the given cache with this one, taking its writes in place of ours.
    pub fn merge(&mut self, other: Cache) {
        // One might ask, why does this exist separately from `apply_to`?  The
        // answer is that `apply_to` takes a `StateWrite`, so we'd have to have
        // `Cache: StateWrite`, and that implies `Cache: StateRead`, but the
        // `StateRead` trait assumes asynchronous access, and in any case, we
        // probably don't want to be reading directly from a `Cache` (?)
        self.unwritten_changes.extend(other.unwritten_changes);
        self.nonverifiable_changes
            .extend(other.nonverifiable_changes);
        self.ephemeral_objects.extend(other.ephemeral_objects);
        self.events.extend(other.events);
    }

    /// Consume this cache, applying its writes to the given state.
    pub fn apply_to<S: StateWrite>(self, mut state: S) {
        for (key, value) in self.unwritten_changes {
            if let Some(value) = value {
                state.put_raw(key, value);
            } else {
                state.delete(key);
            }
        }

        for (key, value) in self.nonverifiable_changes {
            if let Some(value) = value {
                state.nonverifiable_put_raw(key, value);
            } else {
                state.nonverifiable_delete(key);
            }
        }

        // It's important to use object_merge here, so that we don't re-box all
        // of the objects, causing downcasting to fail.
        state.object_merge(self.ephemeral_objects);

        for event in self.events {
            state.record(event);
        }
    }

    /// Returns `true` if there are cached writes on top of the snapshot, and `false` otherwise.
    pub fn is_dirty(&self) -> bool {
        !(self.unwritten_changes.is_empty()
            && self.nonverifiable_changes.is_empty()
            && self.ephemeral_objects.is_empty())
    }

    /// Extracts and returns the ABCI events contained in this cache.
    pub fn take_events(&mut self) -> Vec<abci::Event> {
        std::mem::take(&mut self.events)
    }

    /// Consumes a `Cache` and returns a map of `SubstoreConfig` to `Cache` that
    /// corresponds to changes belonging to each substore. The keys in each `Cache`
    /// are truncated to remove the substore prefix.
    pub fn shard_by_prefix(
        self,
        prefixes: &MultistoreConfig,
    ) -> BTreeMap<Arc<SubstoreConfig>, Self> {
        let mut changes_by_substore = BTreeMap::new();
        for (key, some_value) in self.unwritten_changes.into_iter() {
            let (truncated_key, substore_config) = prefixes.route_key_str(&key);
            changes_by_substore
                .entry(substore_config)
                .or_insert_with(Cache::default)
                .unwritten_changes
                .insert(truncated_key.to_string(), some_value);
        }

        for (key, some_value) in self.nonverifiable_changes {
            let (truncated_key, substore_config) = prefixes.route_key_bytes(&key);
            changes_by_substore
                .entry(substore_config)
                .or_insert_with(Cache::default)
                .nonverifiable_changes
                .insert(truncated_key.to_vec(), some_value);
        }
        changes_by_substore
    }

    pub(crate) fn clone_changes(&self) -> Self {
        Self {
            unwritten_changes: self.unwritten_changes.clone(),
            nonverifiable_changes: self.nonverifiable_changes.clone(),
            ephemeral_objects: Default::default(),
            events: Default::default(),
        }
    }
}
