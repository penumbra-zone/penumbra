use anyhow::Result;
use cnidarium::StateWrite;
use penumbra_num::Amount;
use position::State::*;

use crate::lp::position::{self, Position, State};
use crate::state_key::engine;
use crate::DirectedTradingPair;
use penumbra_proto::{StateReadProto, StateWriteProto};

pub(crate) trait AssetByLiquidityIndex: StateWrite {
    /// Update the base liquidity index, used by the DEX engine during path search.
    ///
    /// # Overview
    /// Given a directed trading pair `A -> B`, the index tracks the amount of
    /// liquidity available to convert the quote asset B, into a base asset A.
    ///
    /// # Index schema
    /// The liquidity index schema is as follow:
    /// - A primary index that maps a "start" asset A (aka. base asset)
    ///   to an "end" asset B (aka. quote asset) ordered by the amount of
    ///   liquidity available for B -> A (not a typo).
    /// - An auxilliary index that maps a directed trading pair `A -> B`
    ///   to the aggregate liquidity for B -> A (used in the primary composite key)
    ///
    /// # Diagram
    ///                                                                     
    ///    Liquidity index:                                                 
    ///    For an asset `A`, surface asset                                  
    ///    `B` with the best liquidity                                      
    ///    score.                                                           
    ///                             ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐     
    ///                                                                     
    ///           ┌──┐              ▼            ┌─────────┐          │     
    ///     ▲     │  │    ┌──────────────────┐   │         │                
    ///     │     │ ─┼───▶│{asset_A}{agg_liq}│──▶│{asset_B}│          │     
    ///     │     ├──┤    └──────────────────┘   │         │                
    ///   sorted  │  │                           └─────────┘          │     
    ///   by agg  │  │                                                      
    ///    liq    ├──┤                                                │     
    ///     │     │  │                                           used in the
    ///     │     ├──┤                                            composite
    ///     │     │  │                                               key    
    ///     │     │  │       Auxiliary look-up index:                 │     
    ///     │     │  │       "Find the aggregate liquidity                 
    ///     │     │  │       per directed trading pair"               │      
    ///     │     │  │       ┌───────┐                           ┌─────────┐
    ///     │     │  │       ├───────┤  ┌──────────────────┐     │         │
    ///     │     │  │       │   ────┼─▶│{asset_A}{asset_B}│────▶│{agg_liq}│
    ///     │     ├──┤       ├───────┤  └──────────────────┘     │         │
    ///     │     │  │       ├───────┤                           └─────────┘
    ///     │     │  │       ├───────┤                                      
    ///     │     │  │       ├───────┤                                      
    ///     │     ├──┤       └───────┘                                      
    ///     │     │  │                                                      
    ///     │     │  │                                                      
    ///     │     └──┘                                                      
    async fn update_asset_by_base_liquidity_index(
        &mut self,
        prev_state: &Option<Position>,
        new_state: &Position,
        id: &position::Id,
    ) -> Result<()> {
        match prev_state {
            Some(prev_state) => match (prev_state.state, new_state.state) {
                // We only want to update the index when we process active positions.
                (Opened, Closed) => {}
                (Opened, Opened) => {}
                _ => return Ok(()),
            },
            None => {}
        }

        let canonical_pair = new_state.phi.pair;
        let pair_ab = DirectedTradingPair::new(canonical_pair.asset_1(), canonical_pair.asset_2());

        let (prev_a, prev_b) = prev_state
            .as_ref()
            .map(|p| {
                (
                    p.reserves_for(pair_ab.start).expect("asset ids match"),
                    p.reserves_for(pair_ab.end).expect("asset ids match"),
                )
            })
            .unwrap_or_else(|| (Amount::zero(), Amount::zero()));

        // A -> B
        self.update_asset_by_base_liquidity_index_inner(id, pair_ab, prev_a, new_state)
            .await?;
        // B -> A
        self.update_asset_by_base_liquidity_index_inner(id, pair_ab.flip(), prev_b, new_state)
            .await?;

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> AssetByLiquidityIndex for T {}

trait Inner: StateWrite {
    async fn update_asset_by_base_liquidity_index_inner(
        &mut self,
        id: &position::Id,
        pair: DirectedTradingPair,
        old_contrib: Amount,
        new_position: &Position,
    ) -> Result<()> {
        let aggregate_key = &engine::routable_assets::lookup_base_liquidity_by_pair(&pair);

        let prev_tally: Amount = self
            .nonverifiable_get(aggregate_key)
            .await?
            .unwrap_or_default();

        // The previous contribution for this position is supplied to us by
        // the caller. This default to zero if the position was just created.
        // We use this to compute a view of the tally that excludes the position
        // we are currently processing (and avoid double-counting).
        let old_contrib = old_contrib;

        // The updated contribution is the total amount of base asset routable
        // from an adjacent asset.
        let new_contrib = new_position
            .reserves_for(pair.start)
            .expect("asset ids should match");

        let new_tally = match new_position.state {
            State::Opened => prev_tally
                .saturating_sub(&old_contrib)
                .saturating_add(&new_contrib),
            State::Closed => prev_tally.saturating_sub(&old_contrib),
            _ => unreachable!("inner impl is guarded"),
        };

        // If the update operation is a no-op, we can skip the update
        // and return early.
        if prev_tally == new_tally {
            tracing::debug!(
                ?prev_tally,
                ?pair,
                ?id,
                "skipping routable asset index update"
            );
            return Ok(());
        }

        // Update the primary and auxiliary indices:
        let old_primary_key = engine::routable_assets::key(&pair.start, prev_tally).to_vec();
        // This could make the `StateDelta` more expensive to scan, but this doesn't show on profiles yet.
        self.nonverifiable_delete(old_primary_key);

        let new_primary_key = engine::routable_assets::key(&pair.start, new_tally).to_vec();
        self.nonverifiable_put(new_primary_key, pair.end);
        tracing::debug!(?pair, ?new_tally, "base liquidity entry updated");

        let auxiliary_key = engine::routable_assets::lookup_base_liquidity_by_pair(&pair).to_vec();
        self.nonverifiable_put(auxiliary_key, new_tally);
        tracing::debug!(
            ?pair,
            "base liquidity heuristic marked directed pair as routable"
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
