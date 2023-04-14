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
    /// Update this entry with the new path, if it's better than the existing one.
    pub fn update(&mut self, new_path: Path<S>) {
        if new_path.price < self.path.price {
            tracing::debug!(new_price = ?new_path.price, old_price = ?self.path.price, "updating path");
            self.spill = Some(std::mem::replace(&mut self.path, new_path));
            self.active = true;
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
