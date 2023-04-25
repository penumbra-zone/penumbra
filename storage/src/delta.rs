use std::{any::Any, sync::Arc};

use futures::StreamExt;
use parking_lot::RwLock;
use tendermint::abci;

use crate::{
    future::{
        CacheFuture, StateDeltaNonconsensusPrefixRawStream, StateDeltaPrefixKeysStream,
        StateDeltaPrefixRawStream,
    },
    Cache, StateRead, StateWrite,
};

/// An arbitrarily-deeply nested stack of delta updates to an underlying state.
///
/// This API allows exploring a tree of possible execution paths concurrently,
/// before finally selecting one and applying it to the underlying state.
///
/// Using this API requires understanding its invariants.
///
/// On creation, `StateDelta::new` takes ownership of a `StateRead + StateWrite`
/// instance, acquiring a "write lock" over the underlying state (since `&mut S`
/// is `StateWrite` if `S: StateWrite`, it's possible to pass a unique
/// reference).
///
/// The resulting `StateDelta` instance is a "leaf" state, and can be used for
/// reads and writes, following the some execution path.
///
/// When two potential execution paths diverge, `delta.fork()` can be used to
/// fork the state update.  The new forked `StateDelta` will include all
/// previous state writes made to the original (and its ancestors).  Any writes
/// made to the original `StateDelta` after `fork()` is called will not be seen
/// by the forked state.
///
/// Finally, after some execution path has been selected, calling
/// `delta.apply()` on one of the possible state updates will commit the changes
/// to the underlying state instance, and invalidate all other delta updates in
/// the same family.  It is a programming error to use the other delta updates
/// after `apply()` has been called, but ideally this should not be a problem in
/// practice: the API is intended to explore a tree of possible execution paths;
/// once one has been selected, the others should be discarded.
#[derive(Debug)]
pub struct StateDelta<S: StateRead> {
    /// The underlying state instance.
    ///
    /// The Arc<_> allows it to be shared between different stacks of delta updates,
    /// and the RwLock<Option<_>> allows it to be taken out when it's time to commit
    /// the changes from one of the stacks.
    state: Arc<RwLock<Option<S>>>,
    /// A stack of intermediate delta updates, with the "top" layers first.
    ///
    /// We store all the layers directly, rather than using a recursive structure,
    /// so that the type doesn't depend on how many layers are involved. We're only
    /// duplicating the Arc<_>, so this should be cheap.
    layers: Vec<Arc<RwLock<Option<Cache>>>>,
    /// The final delta update in the stack, the one we're currently working on.
    /// Storing this separately allows us to avoid lock contention during writes.
    /// In fact, this data shouldn't usually be shared at all; the only reason it's
    /// wrapped this way is so that prefix streams can have 'static lifetimes.
    /// We option-wrap it so it can be chained with the layers; it will never be None.
    leaf_cache: Arc<RwLock<Option<Cache>>>,
}

impl<S: StateRead> StateDelta<S> {
    /// Create a new tree of possible updates to an underlying `state`.
    pub fn new(state: S) -> Self {
        Self {
            state: Arc::new(RwLock::new(Some(state))),
            layers: Vec::default(),
            leaf_cache: Arc::new(RwLock::new(Some(Cache::default()))),
        }
    }

    /// Fork execution, returning a new child state that includes all previous changes.
    pub fn fork(&mut self) -> Self {
        // If we have writes in the leaf cache, we'll move them to a new layer,
        // ensuring that the new child only sees writes made to this state
        // *before* fork was called, and not after.
        //
        // Doing this only when the leaf cache is dirty means that we don't
        // add empty layers in repeated fork() calls without intervening writes.
        if self.leaf_cache.read().as_ref().unwrap().is_dirty() {
            let new_layer = std::mem::replace(
                &mut self.leaf_cache,
                Arc::new(RwLock::new(Some(Cache::default()))),
            );
            self.layers.push(new_layer);
        }

        Self {
            state: self.state.clone(),
            layers: self.layers.clone(),
            leaf_cache: Arc::new(RwLock::new(Some(Cache::default()))),
        }
    }

    pub(crate) fn flatten(self) -> (S, Cache) {
        // Take ownership of the underlying state, immediately invalidating all
        // other delta stacks in the same family.
        let state = self
            .state
            .write()
            .take()
            .expect("apply must be called only once");

        // Flatten the intermediate layers into a single cache, applying them from oldest
        // (bottom) to newest (top), so that newer writes clobber old ones.
        let mut changes = Cache::default();
        for layer in self.layers {
            let cache = layer
                .write()
                .take()
                .expect("cache must not have already been applied");
            changes.merge(cache);
        }
        // Last, apply the changes in the leaf cache.
        changes.merge(self.leaf_cache.write().take().unwrap());

        (state, changes)
    }
}

impl<S: StateRead + StateWrite> StateDelta<S> {
    /// Apply all changes in this branch of the tree to the underlying state,
    /// releasing it back to the caller and invalidating all other branches of
    /// the tree.
    pub fn apply(self) -> (S, Vec<abci::Event>) {
        let (mut state, mut changes) = self.flatten();
        let events = std::mem::take(&mut changes.events);

        // Apply the flattened changes to the underlying state.
        changes.apply_to(&mut state);

        // Finally, return ownership of the state back to the caller.
        (state, events)
    }
}

impl<S: StateRead + StateWrite> StateDelta<Arc<S>> {
    pub fn try_apply(self) -> Result<(S, Vec<abci::Event>), anyhow::Error> {
        let (arc_state, mut changes) = self.flatten();
        let events = std::mem::take(&mut changes.events);

        if let Ok(mut state) = Arc::try_unwrap(arc_state) {
            // Apply the flattened changes to the underlying state.
            changes.apply_to(&mut state);

            // Finally, return ownership of the state back to the caller.
            Ok((state, events))
        } else {
            Err(anyhow::anyhow!("did not have unique ownership of Arc<S>"))
        }
    }
}

impl<S: StateRead> StateRead for StateDelta<S> {
    type GetRawFut = CacheFuture<S::GetRawFut>;
    type PrefixRawStream = StateDeltaPrefixRawStream<S::PrefixRawStream>;
    type PrefixKeysStream = StateDeltaPrefixKeysStream<S::PrefixKeysStream>;
    type NonconsensusPrefixRawStream =
        StateDeltaNonconsensusPrefixRawStream<S::NonconsensusPrefixRawStream>;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        // Check if we have a cache hit in the leaf cache.
        if let Some(entry) = self
            .leaf_cache
            .read()
            .as_ref()
            .unwrap()
            .unwritten_changes
            .get(key)
        {
            return CacheFuture::hit(entry.clone());
        }

        // Iterate through the stack, top to bottom, to see if we have a cache hit.
        for layer in self.layers.iter().rev() {
            if let Some(entry) = layer
                .read()
                .as_ref()
                .expect("delta must not have been applied")
                .unwritten_changes
                .get(key)
            {
                return CacheFuture::hit(entry.clone());
            }
        }

        // If we got here, the key must be in the underlying state or not present at all.
        CacheFuture::miss(
            self.state
                .read()
                .as_ref()
                .expect("delta must not have been applied")
                .get_raw(key),
        )
    }

    fn nonconsensus_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        // Check if we have a cache hit in the leaf cache.
        if let Some(entry) = self
            .leaf_cache
            .read()
            .as_ref()
            .unwrap()
            .nonconsensus_changes
            .get(key)
        {
            return CacheFuture::hit(entry.clone());
        }

        // Iterate through the stack, top to bottom, to see if we have a cache hit.
        for layer in self.layers.iter().rev() {
            if let Some(entry) = layer
                .read()
                .as_ref()
                .expect("delta must not have been applied")
                .nonconsensus_changes
                .get(key)
            {
                return CacheFuture::hit(entry.clone());
            }
        }

        // If we got here, the key must be in the underlying state or not present at all.
        CacheFuture::miss(
            self.state
                .read()
                .as_ref()
                .expect("delta must not have been applied")
                .nonconsensus_get_raw(key),
        )
    }

    fn object_get<T: std::any::Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        // Check if we have a cache hit in the leaf cache.
        if let Some(entry) = self
            .leaf_cache
            .read()
            .as_ref()
            .expect("delta must not have been applied")
            .ephemeral_objects
            .get(key)
        {
            return entry.as_ref().and_then(|v| v.downcast_ref()).cloned();
        }

        // Iterate through the stack, top to bottom, to see if we have a cache hit.
        for layer in self.layers.iter().rev() {
            if let Some(entry) = layer
                .read()
                .as_ref()
                .expect("delta must not have been applied")
                .ephemeral_objects
                .get(key)
            {
                return entry.as_ref().and_then(|v| v.downcast_ref()).cloned();
            }
        }

        // Fall through to the underlying store.
        self.state
            .read()
            .as_ref()
            .expect("delta must not have been applied")
            .object_get(key)
    }

    fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream {
        let underlying = self
            .state
            .read()
            .as_ref()
            .expect("delta must not have been applied")
            .prefix_raw(prefix)
            .peekable();
        StateDeltaPrefixRawStream {
            underlying,
            layers: self.layers.clone(),
            leaf_cache: self.leaf_cache.clone(),
            last_key: None,
            prefix: prefix.to_owned(),
        }
    }

    fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream {
        let underlying = self
            .state
            .read()
            .as_ref()
            .expect("delta must not have been applied")
            .prefix_keys(prefix)
            .peekable();
        StateDeltaPrefixKeysStream {
            underlying,
            layers: self.layers.clone(),
            leaf_cache: self.leaf_cache.clone(),
            last_key: None,
            prefix: prefix.to_owned(),
        }
    }

    fn nonconsensus_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream {
        let underlying = self
            .state
            .read()
            .as_ref()
            .expect("delta must not have been applied")
            .nonconsensus_prefix_raw(prefix)
            .peekable();
        StateDeltaNonconsensusPrefixRawStream {
            underlying,
            layers: self.layers.clone(),
            leaf_cache: self.leaf_cache.clone(),
            last_key: None,
            prefix: prefix.to_vec(),
        }
    }
}

impl<S: StateRead> StateWrite for StateDelta<S> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .unwritten_changes
            .insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .unwritten_changes
            .insert(key, None);
    }

    fn nonconsensus_delete(&mut self, key: Vec<u8>) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .nonconsensus_changes
            .insert(key, None);
    }

    fn nonconsensus_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .nonconsensus_changes
            .insert(key, Some(value));
    }

    fn object_put<T: Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .ephemeral_objects
            .insert(key, Some(Box::new(value)));
    }

    fn object_delete(&mut self, key: &'static str) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .ephemeral_objects
            .insert(key, None);
    }

    fn object_merge(
        &mut self,
        objects: std::collections::BTreeMap<&'static str, Option<Box<dyn Any + Send + Sync>>>,
    ) {
        self.leaf_cache
            .write()
            .as_mut()
            .unwrap()
            .ephemeral_objects
            .extend(objects);
    }

    fn record(&mut self, event: abci::Event) {
        self.leaf_cache.write().as_mut().unwrap().events.push(event)
    }
}

/// Extension trait providing `try_begin_transaction()` on `Arc<StateDelta<S>>`.
pub trait ArcStateDeltaExt: Sized {
    type S: StateRead;
    /// Attempts to begin a transaction on this `Arc<State>`, returning `None` if the `Arc` is shared.
    fn try_begin_transaction(&'_ mut self) -> Option<StateDelta<&'_ mut StateDelta<Self::S>>>;
}

impl<S: StateRead> ArcStateDeltaExt for Arc<StateDelta<S>> {
    type S = S;
    fn try_begin_transaction(&'_ mut self) -> Option<StateDelta<&'_ mut StateDelta<S>>> {
        Arc::get_mut(self).map(StateDelta::new)
    }
}
