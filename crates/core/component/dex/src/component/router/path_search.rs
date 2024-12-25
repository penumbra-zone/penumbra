use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateDelta, StateRead};
use futures::StreamExt;
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::fixpoint::U128x128;
use tap::Tap;
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use crate::component::PositionRead as _;

use super::{Path, PathCache, PathEntry, RoutingParams, SharedPathCache};

#[async_trait]
pub trait PathSearch: StateRead + Clone + 'static {
    /// Find the best route from `src` to `dst` with estimated price strictly less
    /// than `params.price_limit`, also returning the spill price for the next-best
    /// route, if one exists.
    #[instrument(skip(self, params), fields(max_hops = params.max_hops), level = "debug", ret)]
    async fn path_search(
        &self,
        src: asset::Id,
        dst: asset::Id,
        params: RoutingParams,
    ) -> Result<(Option<Vec<asset::Id>>, Option<U128x128>)> {
        let RoutingParams {
            max_hops,
            fixed_candidates,
            price_limit,
        } = params;

        // Initialize some metrics for calculating time spent on path searching
        // vs route filling. We use vecs so we can count across iterations of the loop.
        tracing::debug!(?src, ?dst, ?max_hops, "searching for path");
        let path_start = std::time::Instant::now();
        let record_duration = || {
            use crate::component::metrics::DEX_PATH_SEARCH_DURATION;
            let elapsed = path_start.elapsed();
            metrics::histogram!(DEX_PATH_SEARCH_DURATION).record(elapsed);
        };

        // Work in a new stack of state changes, which we can completely discard
        // at the end of routing
        let state = StateDelta::new(self.clone());

        let cache = PathCache::begin(src, state);
        for i in 0..max_hops {
            relax_active_paths(cache.clone(), fixed_candidates.clone()).await?;
            tracing::trace!(i, "finished relaxing all active paths");
        }

        let entry = cache.lock().0.remove(&dst);
        let Some(PathEntry { path, spill, .. }) = entry else {
            record_duration();
            return Ok((None, None));
        };

        let nodes = path.nodes;
        let spill_price = spill.map(|p| p.price);
        tracing::debug!(price = %path.price, spill_price = %spill_price.unwrap_or_else(|| 0u64.into()), ?src, ?nodes, "found path");
        record_duration();

        match price_limit {
            // Note: previously, this branch was a load-bearing termination condition, primarily
            // exercised by the arbitrage logic. However, during the course of testnet 53,  we
            // encountered two bugs that caused this predicate to not be exercised:
            // 1. We treated the price limit as an inclusive bound, rather than an exclusive bound.
            // 2. We relied on an estimate of the end-to-end path price which was lossy (`path.price`).
            // The latter is an inherent information limitation, so we now have a redundant check in
            // `route_and_fill` which uses the exact price of the route.
            Some(price_limit) if path.price >= price_limit => {
                tracing::debug!(price = %path.price, price_limit = %price_limit, "path too expensive");
                Ok((None, None))
            }
            _ => Ok((Some(nodes), spill_price)),
        }
    }
}

impl<S> PathSearch for S where S: StateRead + Clone + 'static {}

#[instrument(skip_all)]
async fn relax_active_paths<S: StateRead + 'static>(
    cache: SharedPathCache<S>,
    fixed_candidates: Arc<Vec<asset::Id>>,
) -> Result<()> {
    let active_paths = cache.lock().extract_active();
    let mut js = JoinSet::new();
    tracing::trace!(
        active_paths_len = active_paths.len(),
        "relaxing active paths"
    );
    for path in active_paths {
        let candidates = Arc::clone(&fixed_candidates);
        let cache = Arc::clone(&cache);
        js.spawn(async move {
            use crate::component::metrics::DEX_PATH_SEARCH_RELAX_PATH_DURATION;
            let metric = metrics::histogram!(DEX_PATH_SEARCH_RELAX_PATH_DURATION);
            let start = std::time::Instant::now();
            relax_path(cache, path, candidates)
                .await
                .tap(|_| metric.record(start.elapsed()))
        });
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
    fixed_candidates: Arc<Vec<asset::Id>>,
) -> Result<()> {
    let mut candidates = path
        .state
        .candidate_set(*path.end(), fixed_candidates)
        .instrument(path.span.clone());

    path.span.in_scope(|| {
        tracing::trace!("relaxing path");
    });

    let mut js = JoinSet::new();

    while let Some(new_end) = candidates.inner_mut().next().await {
        let new_path = path.fork();
        let cache2 = cache.clone();
        js.spawn(async move {
            if let Some(new_path) = new_path.extend_to(new_end?).await? {
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
