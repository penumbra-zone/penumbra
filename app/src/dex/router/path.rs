use anyhow::Result;

use penumbra_crypto::{asset, dex::DirectedTradingPair, fixpoint::U128x128};
use penumbra_storage::{StateDelta, StateRead};
use tracing::Instrument;

use super::super::PositionRead;

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

    pub fn state(&self) -> &StateDelta<S> {
        &self.state
    }

    // We can't clone, because StateDelta only has an explicit fork() on purpose
    pub fn fork(&mut self) -> Self {
        self.span.in_scope(|| tracing::debug!("forking path"));
        Self {
            start: self.start.clone(),
            nodes: self.nodes.clone(),
            price: self.price.clone(),
            state: self.state.fork(),
            span: self.span.clone(),
        }
    }

    // Making this consuming forces callers to explicitly fork the path first.
    pub async fn extend_to(mut self, new_end: asset::Id) -> Result<Option<Path<S>>> {
        let span = tracing::debug_span!(parent: &self.span, "extend_to", new_end = ?new_end);
        // Passing to an inner function lets us control the span more precisely than if
        // we used the #[instrument] macro (which does something similar to this internally).
        self.extend_to_inner(new_end).instrument(span).await
    }

    async fn extend_to_inner(mut self, new_end: asset::Id) -> Result<Option<Path<S>>> {
        let target_pair = DirectedTradingPair::new(self.end().clone(), new_end.clone());
        let Some(best_price_position) = self.state.best_position(&target_pair).await? else {
            tracing::debug!("no best position, failing to extend path");
            return Ok(None)
        };
        // Deindex the position we "consumed" in this and all descendant state forks,
        // ensuring we don't double-count liquidity while traversing cycles.
        use super::super::position_manager::Inner as _;
        self.state.deindex_position(&best_price_position);

        // Update and return the path.
        // TODO: gross
        let hop_price = if self.end() == &best_price_position.phi.pair.asset_1() {
            best_price_position.phi.component.effective_price()
        } else {
            best_price_position.phi.component.flip().effective_price()
        };

        if let Some(path_price) = self.price * hop_price {
            tracing::debug!(%path_price, %hop_price, id = ?best_price_position.id(), "extended path");
            self.price = path_price;
            self.nodes.push(new_end);
            // Create a new span for the extension.  Note: this is a child of
            // the path span (:path:via:via:via etc), not a child of the current
            // span (:path:via:via:extend_to).
            self.span = tracing::debug_span!(parent: &self.span, "via", id = ?new_end);
            Ok(Some(self))
        } else {
            // If there was an overflow estimating the effective price, we failed
            // to extend the path.
            tracing::debug!("failed to extend path due to overflow");
            Ok(None)
        }
    }
}
