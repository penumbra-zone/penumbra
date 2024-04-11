use anyhow::Result;
use cnidarium::StateWrite;
use penumbra_num::Amount;

use crate::lp::position::{Position, State};
use crate::state_key::engine;
use crate::DirectedTradingPair;
use penumbra_proto::DomainType;

pub(crate) trait AssetByLiquidityIndex: StateWrite {
    /// Updates the nonverifiable liquidity indices given a [`Position`] in the direction specified by the [`DirectedTradingPair`].
    /// An [`Option<Position>`] may be specified to allow for the case where a position is being updated.
    async fn update_liquidity_index(
        &mut self,
        pair: DirectedTradingPair,
        position: &Position,
        prev: &Option<Position>,
    ) -> Result<()> {
        tracing::debug!(?pair, "updating available liquidity indices");

        let (new_a_from_b, current_a_from_b) = match (position.state, prev) {
            (State::Opened, None) => {
                // Add the new position's contribution to the index, no cancellation of the previous version necessary.

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&engine::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the new reserves to compute `new_position_contribution`,
                // the amount of asset A contributed by the position (i.e. the reserves of asset A).
                let new_position_contribution = position
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                    // Add the contribution from the updated version.
                    current_a_from_b.saturating_add(&new_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?new_position_contribution, "newly opened position, adding contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Opened, Some(prev)) => {
                // Add the new position's contribution to the index, deleting the previous version's contribution.

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&engine::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the previous reserves to compute `prev_position_contribution` (denominated in asset_1).
                let prev_position_contribution = prev
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Use the new reserves to compute `new_position_contribution`,
                // the amount of asset A contributed by the position (i.e. the reserves of asset A).
                let new_position_contribution = position
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                // Subtract the previous version of the position's contribution to represent that position no longer
                // being correct, and add the contribution from the updated version.
                (current_a_from_b.saturating_sub(&prev_position_contribution)).saturating_add(&new_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?new_position_contribution, ?prev_position_contribution, "updated position, adding new contribution and subtracting previous contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Closed, Some(prev)) => {
                // Compute the previous contribution and erase it from the current index

                // Query the current available liquidity for this trading pair, or zero if the trading pair
                // has no current liquidity.
                let current_a_from_b = self
                    .nonverifiable_get_raw(&engine::routable_assets::a_from_b(&pair))
                    .await?
                    .map(|bytes| {
                        Amount::from_be_bytes(
                            bytes
                                .try_into()
                                .expect("liquidity index amount can always be parsed"),
                        )
                    })
                    .unwrap_or_default();

                // Use the previous reserves to compute `prev_position_contribution` (denominated in asset_1).
                let prev_position_contribution = prev
                    .reserves_for(pair.start)
                    .expect("specified position should match provided trading pair");

                // Compute `new_A_from_B`.
                let new_a_from_b =
                // Subtract the previous version of the position's contribution to represent that position no longer
                // being correct, and since the updated version is Closed, it has no contribution.
                current_a_from_b.saturating_sub(&prev_position_contribution);

                tracing::debug!(?pair, current_liquidity = ?current_a_from_b, ?prev_position_contribution, "closed position, subtracting previous contribution to existing available liquidity for trading pair");

                (new_a_from_b, current_a_from_b)
            }
            (State::Withdrawn { .. }, _) | (State::Closed, None) => {
                // The position already went through the `Closed` state or was opened in the `Closed` state, so its contribution has already been subtracted.
                return Ok(());
            }
        };

        // Delete the existing key for this position if the reserve amount has changed.
        if new_a_from_b != current_a_from_b {
            self.nonverifiable_delete(
                engine::routable_assets::key(&pair.start, current_a_from_b).to_vec(),
            );
        }

        // Write the new key indicating that asset B is routable from asset A with `new_a_from_b` liquidity.
        self.nonverifiable_put_raw(
            engine::routable_assets::key(&pair.start, new_a_from_b).to_vec(),
            pair.end.encode_to_vec(),
        );
        tracing::debug!(start = ?pair.start, end = ?pair.end, "marking routable from start -> end");

        // Write the new lookup index storing `new_a_from_b` for this trading pair.
        self.nonverifiable_put_raw(
            engine::routable_assets::a_from_b(&pair).to_vec(),
            new_a_from_b.to_be_bytes().to_vec(),
        );
        tracing::debug!(available_liquidity = ?new_a_from_b, ?pair, "marking available liquidity for trading pair");

        Ok(())
    }

    async fn update_available_liquidity(
        &mut self,
        prev_position: &Option<Position>,
        position: &Position,
    ) -> Result<()> {
        // Since swaps may be performed in either direction, the available liquidity indices
        // need to be calculated and stored for both the A -> B and B -> A directions.
        let (a, b) = (position.phi.pair.asset_1(), position.phi.pair.asset_2());

        // A -> B
        self.update_liquidity_index(DirectedTradingPair::new(a, b), position, prev_position)
            .await?;
        // B -> A
        self.update_liquidity_index(DirectedTradingPair::new(b, a), position, prev_position)
            .await?;

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> AssetByLiquidityIndex for T {}
