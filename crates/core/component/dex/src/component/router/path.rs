use anyhow::Result;

use cnidarium::{StateDelta, StateRead};
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::fixpoint::U128x128;
use std::cmp::Ordering;
use tracing::Instrument;

use crate::{component::PositionRead, DirectedTradingPair};

/// A path is an ordered sequence of assets, implicitly defining a trading pair,
/// and a price for trading along that path. It contains a forked view of the
/// state after traveling along the path.
///
/// # Ordering
/// The ordering of paths is based on their effective price estimate first,
/// then their length, then their start asset, and finally their intermediary
/// assets.
pub(super) struct Path<S: StateRead + 'static> {
    /// The start point of the path
    pub start: asset::Id,
    /// The nodes along the path, implicitly defining the end
    pub nodes: Vec<asset::Id>,
    /// An estimate of the end-to-end effective price along the path
    pub price: U128x128,
    /// A forked view of the state after traveling along this path.
    pub state: StateDelta<S>,
    /// A span recording information about the path, for debugging.
    pub span: tracing::Span,
}

impl<S: StateRead + 'static> Path<S> {
    pub fn end(&self) -> &asset::Id {
        self.nodes.last().unwrap_or(&self.start)
    }

    pub fn begin(start: asset::Id, state: StateDelta<S>) -> Self {
        let span = tracing::debug_span!("path", start = ?start);
        span.in_scope(|| tracing::debug!("beginning path"));
        Self {
            start,
            nodes: Vec::new(),
            price: 1u64.into(),
            state,
            span,
        }
    }

    // We can't clone, because StateDelta only has an explicit fork() on purpose
    pub fn fork(&mut self) -> Self {
        Self {
            start: self.start,
            nodes: self.nodes.clone(),
            price: self.price,
            state: self.state.fork(),
            span: self.span.clone(),
        }
    }

    // Making this consuming forces callers to explicitly fork the path first.
    pub async fn extend_to(self, new_end: asset::Id) -> Result<Option<Path<S>>> {
        let span = tracing::debug_span!(parent: &self.span, "extend_to", new_end = ?new_end);
        // Passing to an inner function lets us control the span more precisely than if
        // we used the #[instrument] macro (which does something similar to this internally).
        self.extend_to_inner(new_end).instrument(span).await
    }

    async fn extend_to_inner(mut self, new_end: asset::Id) -> Result<Option<Path<S>>> {
        let target_pair = DirectedTradingPair::new(*self.end(), new_end);
        // Pulls the (id, position) that have the best effective price for this hop.
        let Some((best_price_lp_id, best_price_lp)) =
            self.state.best_position(&target_pair).await?
        else {
            tracing::trace!("no best position, failing to extend path");
            return Ok(None);
        };
        // Deindex the position we "consumed" in this and all descendant state forks,
        // ensuring we don't double-count liquidity while traversing cycles.
        use crate::component::position_manager::price_index::PositionByPriceIndex;
        self.state
            .deindex_position_by_price(&best_price_lp, &best_price_lp_id);

        // Compute the effective price of a trade in the direction self.end()=>new_end
        let hop_price = best_price_lp
            .phi
            .orient_end(new_end)
            .expect("position should be contain the end asset")
            .effective_price();

        match self.price * hop_price {
            Ok(path_price) => {
                // Update and return the path.
                tracing::debug!(%path_price, %hop_price, ?best_price_lp_id, "extended path");
                self.price = path_price;
                self.nodes.push(new_end);
                // Create a new span for the extension.  Note: this is a child of
                // the path span (:path:via:via:via etc), not a child of the current
                // span (:path:via:via:extend_to).
                self.span = tracing::debug_span!(parent: &self.span, "via", id = ?new_end);
                Ok(Some(self))
            }
            Err(e) => {
                // If there was an overflow estimating the effective price, we failed
                // to extend the path.
                tracing::debug!(?e, "failed to extend path due to overflow");
                Ok(None)
            }
        }
    }
}

impl<S: StateRead + 'static> PartialEq for Path<S> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.price == other.price && self.nodes == other.nodes
    }
}

impl<S: StateRead + 'static> PartialOrd for Path<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: StateRead + 'static> Eq for Path<S> {}

impl<S: StateRead + 'static> Ord for Path<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price).then_with(|| {
            self.nodes.len().cmp(&other.nodes.len()).then_with(|| {
                self.start
                    .cmp(&other.start)
                    .then_with(|| self.nodes.cmp(&other.nodes))
            })
        })
    }
}
