use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::{any::Any, cmp::Ordering, collections::BTreeMap, iter::Peekable, pin::Pin};

use crate::State;

use super::{read::prefix_raw_with_cache, StateRead, StateWrite};

/// A set of pending changes to a [`State`] instance, supporting both writes and reads.
pub struct Transaction<'a> {
    /// Unwritten changes to the consensus-critical state (stored in the JMT).
    pub(crate) unwritten_changes: BTreeMap<String, Option<Vec<u8>>>,
    /// Unwritten changes to non-consensus-critical state (stored in the nonconsensus storage).
    pub(crate) nonconsensus_changes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Unwritten changes to the object store.  A `None` value means a deletion.
    pub(crate) object_changes: BTreeMap<String, Option<Box<dyn Any + Send + Sync>>>,
    state: &'a mut State,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(state: &'a mut State) -> Self {
        Self {
            state,
            unwritten_changes: BTreeMap::new(),
            nonconsensus_changes: BTreeMap::new(),
            object_changes: BTreeMap::new(),
        }
    }

    /// Applies this transaction's writes to its parent [`State`], completing the transaction.
    pub fn apply(self) {
        // Write the unwritten consensus-critical changes to the state:
        self.state.unwritten_changes.extend(self.unwritten_changes);

        // Write the unwritten nonconsensus changes to the state:
        self.state
            .nonconsensus_changes
            .extend(self.nonconsensus_changes);

        for (k, v_or_deletion) in self.object_changes {
            match v_or_deletion {
                Some(v) => {
                    self.state.ephemeral_objects.insert(k, v);
                }
                None => {
                    self.state.ephemeral_objects.remove(&k);
                }
            }
        }
    }

    /// Aborts this transaction, discarding its writes.
    ///
    /// Convienence method for [`std::mem::drop`].
    pub fn abort(self) {}
}

impl<'a> StateWrite for Transaction<'a> {
    fn put_raw(&mut self, key: String, value: jmt::OwnedValue) {
        self.unwritten_changes.insert(key, Some(value));
    }

    fn delete(&mut self, key: String) {
        self.unwritten_changes.insert(key, None);
    }

    fn delete_nonconsensus(&mut self, key: Vec<u8>) {
        self.nonconsensus_changes.insert(key, None);
    }

    fn put_nonconsensus(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nonconsensus_changes.insert(key, Some(value));
    }

    fn put_ephemeral<T: Any + Send + Sync>(&mut self, key: String, value: T) {
        self.object_changes.insert(key, Some(Box::new(value)));
    }

    fn delete_ephemeral(&mut self, key: String) {
        self.object_changes.insert(key, None);
    }
}

//#[async_trait(?Send)]
#[async_trait]
impl<'tx> StateRead for Transaction<'tx> {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // If the key is available in the unwritten_changes cache, return it.
        if let Some(v) = self.unwritten_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the state, return it.
        self.state.get_raw(key).await
    }

    async fn get_nonconsensus(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // If the key is available in the nonconsensus cache, return it.
        if let Some(v) = self.nonconsensus_changes.get(key) {
            return Ok(v.clone());
        }

        // Otherwise, if the key is available in the state, return it.
        self.state.get_nonconsensus(key).await
    }

    fn prefix_raw<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<(String, Vec<u8>)>> + Sync + Send + 'a>> {
        prefix_raw_with_cache(self.state, &self.unwritten_changes, prefix)
    }

    fn get_ephemeral<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        if let Some(v_or_deletion) = self.object_changes.get(key) {
            return v_or_deletion.as_ref().and_then(|v| v.downcast_ref());
        }
        self.state.get_ephemeral(key)
    }

    fn prefix_ephemeral<'a, T: Any + Send + Sync>(
        &'a self,
        prefix: &'a str,
    ) -> Box<dyn Iterator<Item = (&'a str, &'a T)> + 'a> {
        let changes: Box<dyn Iterator<Item = (&'a str, Option<&'a T>)>> = Box::new(
            self.object_changes
                .range(prefix.to_string()..)
                .take_while(move |(k, _)| k.starts_with(prefix))
                // We want changes to always cover the underlying store, so
                // we treat a failed downcast_ref as a deletion.
                .map(
                    |(k, v)| match v.as_ref().and_then(|v| v.downcast_ref::<T>()) {
                        Some(v) => (k.as_str(), Some(v)),
                        None => (k.as_str(), None),
                    },
                ),
        );

        let changes = changes.peekable();
        let underlying = self.state.prefix_ephemeral(prefix).peekable();

        Box::new(MergedObjectIterator {
            changes,
            underlying,
        })
    }
}

struct MergedObjectIterator<'a, T: Any + Send + Sync> {
    /// We want changes to always cover the underlying store, so we don't want to have
    /// already pre-filtered with downcast_ref.
    changes: Peekable<Box<dyn Iterator<Item = (&'a str, Option<&'a T>)> + 'a>>,
    underlying: Peekable<Box<dyn Iterator<Item = (&'a str, &'a T)> + 'a>>,
}

impl<'a, T: Any + Send + Sync> Iterator for MergedObjectIterator<'a, T> {
    type Item = (&'a str, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.changes.peek(), self.underlying.peek()) {
                (Some(c), Some(u)) => {
                    // Use key ordering to determine which item to use next
                    match c.0.cmp(u.0) {
                        Ordering::Less => {
                            // Draw from changes.
                            match self.changes.next().expect("already peeked") {
                                // The key is present, so yield it.
                                (k, Some(v)) => return Some((k, v)),
                                // The key has been deleted, so we want to skip it and continue merging.
                                (_, None) => continue,
                            }
                        }
                        Ordering::Equal => {
                            // We need to advance both iterators, because we want to return only one
                            // item per *distinct* key, with the `changes` shadowing the `underlying`.
                            // Otherwise, we'd return the underlying value in the next iteration.
                            let _ = self.underlying.next();
                            match self.changes.next().expect("already peeked") {
                                // The key is present, so yield it.
                                (k, Some(v)) => return Some((k, v)),
                                // The key has been deleted, so we want to skip it and continue merging.
                                (_, None) => continue,
                            }
                        }
                        Ordering::Greater => {
                            return Some(self.underlying.next().expect("already peeked"))
                        }
                    }
                }
                (Some(_changed), None) => {
                    // Draw from changes.
                    match self.changes.next().expect("already peeked") {
                        // The key is present, so yield it.
                        (k, Some(v)) => return Some((k, v)),
                        // The key has been deleted, so we want to skip it and continue merging.
                        (_, None) => continue,
                    }
                }
                (None, Some(_underlying)) => {
                    return Some(self.underlying.next().expect("already peeked"))
                }
                (None, None) => return None,
            }
        }
    }
}
