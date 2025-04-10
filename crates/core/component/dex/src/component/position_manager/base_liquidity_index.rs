use anyhow::Result;
use cnidarium::StateWrite;
use penumbra_sdk_num::Amount;
use position::State::*;
use tracing::instrument;

use crate::lp::position::{self, Position};
use crate::state_key::engine;
use crate::DirectedTradingPair;
use async_trait::async_trait;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

#[async_trait]
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
    /// - An auxiliary index that maps a directed trading pair `A -> B`
    ///   to the aggregate liquidity for B -> A (used in the primary composite key)
    ///
    /// If we want liquidity rankings for assets adjacent to A, the ranking has to be
    /// denominated in asset A, since that’s the only way to get commensurability when
    /// ranking B C D E etc.
    ///
    /// There are then two possible amounts to consider for an asset B: amount of A that
    /// can be sold for B and amount of A that can be bought with B
    ///
    /// (1), amount that can be sold (“outbound”) is the wrong thing to use
    /// (2), amount that can be bought, is intuitively the “opposite” of what we want,
    ///      since it’s the reverse direction, but is actually the right thing to use as
    ///      a rough proxy for liquidity
    ///
    /// The reason is that (1) can be easily manipulated without any skin in the game, by
    /// offering to sell a tiny amount of B for A at an outrageous/infinite price.
    ///
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
        id: &position::Id,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) -> Result<()> {
        // We need to reconstruct the position's previous contribution and compute
        // its new contribution to the index. We do this for each asset in the pair
        // and short-circuit if all contributions are zero.
        let canonical_pair = new_state.phi.pair;
        let pair_ab = DirectedTradingPair::new(canonical_pair.asset_1(), canonical_pair.asset_2());

        // We reconstruct the position's *previous* contribution so that we can deduct them later:
        let (prev_a, prev_b) = match prev_state {
            // The position was just created, so its previous contributions are zero.
            None => (Amount::zero(), Amount::zero()),
            Some(prev) => match prev.state {
                // The position was previously closed or withdrawn, so its previous contributions are zero.
                Closed | Withdrawn { sequence: _ } => (Amount::zero(), Amount::zero()),
                // The position's previous contributions are the reserves for the start and end assets.
                _ => (
                    prev.reserves_for(pair_ab.start)
                        .expect("asset ids match for start"),
                    prev.reserves_for(pair_ab.end)
                        .expect("asset ids match for end"),
                ),
            },
        };

        // For each asset, we compute the new position's contribution to the index:
        let (new_a, new_b) = if matches!(new_state.state, Closed | Withdrawn { sequence: _ }) {
            // The position is being closed or withdrawn, so its new contributions are zero.
            // Note a withdrawn position MUST have zero reserves, so hardcoding this is extra.
            (Amount::zero(), Amount::zero())
        } else {
            (
                // The new amount of asset A:
                new_state
                    .reserves_for(pair_ab.start)
                    .expect("asset ids match for start"),
                // The new amount of asset B:
                new_state
                    .reserves_for(pair_ab.end)
                    .expect("asset ids match for end"),
            )
        };

        // If all contributions are zero, we can skip the update.
        // This can happen if we're processing inactive transitions like `Closed -> Withdrawn`.
        if prev_a == Amount::zero()
            && new_a == Amount::zero()
            && prev_b == Amount::zero()
            && new_b == Amount::zero()
        {
            return Ok(());
        }

        // A -> B
        self.update_asset_by_base_liquidity_index_inner(id, pair_ab, prev_a, new_a)
            .await?;
        // B -> A
        self.update_asset_by_base_liquidity_index_inner(id, pair_ab.flip(), prev_b, new_b)
            .await?;

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> AssetByLiquidityIndex for T {}

trait Inner: StateWrite {
    #[instrument(skip(self))]
    async fn update_asset_by_base_liquidity_index_inner(
        &mut self,
        id: &position::Id,
        pair: DirectedTradingPair,
        old_contrib: Amount,
        new_contrib: Amount,
    ) -> Result<()> {
        let aggregate_key = &engine::routable_assets::lookup_base_liquidity_by_pair(&pair);

        let prev_tally: Amount = self
            .nonverifiable_get(aggregate_key)
            .await?
            .unwrap_or_default();

        // To compute the new aggregate liquidity, we deduct the old contribution
        // and add the new contribution. We use saturating arithmetic defensively.
        let new_tally = prev_tally
            .saturating_sub(&old_contrib)
            .saturating_add(&new_contrib);

        // If the update operation is a no-op, we can skip the update and return early.
        if prev_tally == new_tally {
            tracing::trace!(
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
        tracing::trace!(
            ?pair,
            "base liquidity heuristic marked directed pair as routable"
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
