use std::{collections::BTreeMap, sync::Arc};

use parking_lot::Mutex;
use penumbra_crypto::asset;
use penumbra_storage::{StateDelta, StateRead};

use super::Path;

/// An entry in the path cache, representing a best known sub-path.
pub(super) struct PathEntry<S: StateRead + 'static> {
    /// The best known path to the intermediate asset.
    pub path: Path<S>,
    /// Whether the path is active, used to implement the SPFA optimization.
    /// Newly extended paths are active.  By deactivating paths if all of their
    /// possible extensions are suboptimal, we can avoid re-extending them in
    /// each iteration.
    pub active: bool,
    /// The second best known path to the intermediate asset, used to record spill prices.
    pub spill: Option<Path<S>>,
}

impl<S: StateRead + 'static> PathEntry<S> {
    /// Update the best path or spill price if the new path is better, otherwise do nothing.
    pub fn update(&mut self, new_path: Path<S>) {
        if new_path < self.path {
            tracing::debug!(new_price = %new_path.price, old_price = %self.path.price, "new path is better than best path, updating cache");
            self.spill = Some(std::mem::replace(&mut self.path, new_path));
            self.active = true;
        } else {
            // The new path is worse than the best path, but it might be better than the spill path.
            self.update_spill(new_path);
        }
    }

    /// Update the spill price if the new path is better, or if the spill price has not
    /// been set yet. Otherwise do nothing.
    fn update_spill(&mut self, new_path: Path<S>) {
        match &self.spill {
            Some(spill) if new_path.price < spill.price => {
                tracing::debug!(new_spill_price = %new_path.price, old_spill_price = %spill.price, "new path is better than spill path, updating cache");
                self.spill = Some(new_path);
                self.active = true;
            }
            Some(spill) => {
                tracing::debug!(new_spill_price = %new_path.price, old_spill_price = %spill.price, "new path is worse than spill path, ignore");
            }
            None => {
                tracing::debug!(new_spill_price = %new_path.price, "new path is a suitable spill path, updating cache");
                self.spill = Some(new_path);
                self.active = true;
            }
        }
    }
}

impl<S: StateRead + 'static> From<Path<S>> for PathEntry<S> {
    fn from(path: Path<S>) -> Self {
        Self {
            path,
            active: true,
            spill: None,
        }
    }
}

pub(super) struct PathCache<S: StateRead + 'static>(pub(super) BTreeMap<asset::Id, PathEntry<S>>);
pub(super) type SharedPathCache<S> = Arc<Mutex<PathCache<S>>>;

impl<S: StateRead + 'static> PathCache<S> {
    /// Initializes a new PathCache with the identity path for the start asset.
    pub fn begin(start: asset::Id, state: StateDelta<S>) -> SharedPathCache<S> {
        let mut cache = BTreeMap::new();
        cache.insert(
            start.clone(),
            PathEntry {
                path: Path::begin(start, state),
                active: true,
                spill: None,
            },
        );
        Arc::new(Mutex::new(Self(cache)))
    }

    /// Consider a new candidate path, updating if it's better than an existing one.
    pub fn consider(&mut self, path: Path<S>) {
        // We can't use the entry combinators because avoiding cloning requires
        // establishing that we'll only do one of the two operations.
        use std::collections::btree_map::Entry;
        let span = path.span.clone();
        span.in_scope(|| match self.0.entry(*path.end()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().update(path);
            }
            Entry::Vacant(entry) => {
                tracing::debug!("inserting new path");
                entry.insert(path.into());
            }
        })
    }

    /// Extract all active paths, marking their existing entries as inactive.
    pub fn extract_active(&mut self) -> Vec<Path<S>> {
        self.0
            .iter_mut()
            .filter_map(|(_, entry)| {
                if entry.active {
                    entry.active = false;
                    Some(entry.path.fork())
                } else {
                    None
                }
            })
            .collect()
    }
}
