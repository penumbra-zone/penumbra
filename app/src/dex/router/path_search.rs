use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{asset, fixpoint::U128x128};
use penumbra_storage::{StateDelta, StateRead};
use tokio::task::JoinSet;
use tracing::instrument;

use super::{Path, PathCache, PathEntry, SharedPathCache};

#[async_trait]
pub trait PathSearch: StateRead + Clone + 'static {
    /// Find the best route from `src` to `dst`, also returning the spill price for the next-best route, if one exists.
    #[instrument(skip(self, src, dst))]
    async fn path_search(
        &self,
        src: asset::Id,
        dst: asset::Id,
        max_hops: usize,
    ) -> Result<(Option<Vec<asset::Id>>, Option<U128x128>)> {
        // Work in a new stack of state changes, which we can completely discard
        // at the end of routing
        let state = StateDelta::new(self.clone());

        let cache = PathCache::begin(src, state);
        for i in 0..max_hops {
            relax_active_paths(cache.clone()).await?;
            tracing::debug!(i, "finished relaxing all active paths");
        }

        let entry = cache.lock().0.remove(&dst);
        if let Some(PathEntry { path, spill, .. }) = entry {
            let nodes = path.nodes;
            let spill_price = spill.map(|p| p.price);
            Ok((Some(nodes), spill_price))
        } else {
            Ok((None, None))
        }
    }
}

impl<S> PathSearch for S where S: StateRead + Clone + 'static {}

async fn relax_active_paths<S: StateRead + 'static>(cache: SharedPathCache<S>) -> Result<()> {
    let active_paths = cache.lock().extract_active();
    let mut js = JoinSet::new();
    for path in active_paths {
        js.spawn(relax_path(cache.clone(), path));
    }
    // Wait for all relaxations to complete.
    while let Some(task) = js.join_next().await {
        task??;
    }
    Ok(())
}

async fn relax_path<S: StateRead + 'static>(
    cache: SharedPathCache<S>,
    mut path: Path<S>,
) -> Result<()> {
    // TODO: replace
    let candidates = hardcoded_candidates();

    let mut js = JoinSet::new();
    for new_end in candidates {
        let new_path = path.fork();
        let cache2 = cache.clone();
        js.spawn(async move {
            if let Some(new_path) = new_path.extend_to(new_end).await? {
                cache2.lock().consider(new_path)
            }
            anyhow::Ok(())
        });
    }
    // Wait for all candidates to be considered.
    while let Some(task) = js.join_next().await {
        task??;
    }
    Ok(())
}

fn hardcoded_candidates() -> Vec<asset::Id> {
    vec![
        asset::REGISTRY.parse_unit("gm").id(),
        asset::REGISTRY.parse_unit("gn").id(),
        asset::REGISTRY.parse_unit("pusd").id(),
        asset::REGISTRY.parse_unit("penumbra").id(),
    ]
}
