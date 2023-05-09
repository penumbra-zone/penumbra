use std::collections::{BTreeSet, HashMap};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use penumbra_crypto::{asset, dex::DirectedTradingPair, fixpoint::U128x128, Amount, Value};
use penumbra_storage::StateWrite;
use tracing::instrument;

use crate::dex::{PositionManager, PositionRead};

#[async_trait]
pub trait FillRoute2: StateWrite {
    ///
    ///
    /// # Invariants
    ///
    /// It is an error to call `fill_route` on a route that does not have at least one position for each hop.
    #[instrument(skip(self, input, hops, spill_price))]
    async fn fill_route(
        &mut self,
        mut input: Value,
        hops: &[asset::Id],
        spill_price: Option<U128x128>,
    ) -> Result<(Value, Value)> {
        // Switch from representing hops implicitly as a sequence of asset IDs to
        // representing them explicitly as a sequence of directed trading pairs.
        let route = std::iter::once(input.asset_id)
            .chain(hops.iter().cloned())
            .collect::<Vec<_>>();
        // Break down the route into a sequence of pairs to visit.
        let pairs = breakdown_route(&route)?;

        tracing::debug!(
            input = ?input.amount,
            ?route,
            ?spill_price,
        );

        // We want to ensure that any particular position is used at most once over the route,
        // even if the route has cycles at the macro-scale. To do this, we store the streams
        // of positions for each pair, taking care to only construct one stream per distinct pair.
        let mut positions_by_price = HashMap::new();
        for pair in &pairs {
            positions_by_price
                .entry(pair.clone())
                .or_insert_with(|| self.positions_by_price(&pair));
        }

        // Record a trace of the execution along the current route,
        // starting with all-zero amounts.
        let mut trace: Vec<Value> = std::iter::once(Value {
            amount: 0u64.into(),
            asset_id: input.asset_id,
        })
        .chain(hops.iter().map(|asset_id| Value {
            amount: 0u64.into(),
            asset_id: asset_id.clone(),
        }))
        .collect();

        let mut output = Value {
            amount: 0u64.into(),
            asset_id: route
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("called fill_route with empty route"))?,
        };

        let mut frontier = Vec::new();

        // Tracks a set of position IDs currently used in the frontier.
        //
        // This lets us correctly handle the case where we traverse the same macro-edge
        // in opposite directions, and a position has nonzero reserves of both assets
        // and shows up in both position streams (even though we must only use it once).
        let mut frontier_position_ids = BTreeSet::new();

        for pair in &pairs {
            'next_position: {
                let id = positions_by_price
                    .get_mut(pair)
                    .unwrap()
                    .as_mut()
                    .next()
                    .await
                    .ok_or_else(|| {
                        anyhow!("no positions available for pair on path {:?}", pair)
                    })??;

                // Check that the position is not already part of the frontier.
                if !frontier_position_ids.contains(&id) {
                    let position = self
                        .position_by_id(&id)
                        .await?
                        .ok_or_else(|| anyhow!("position with indexed id {:?} not found", id))?;

                    frontier_position_ids.insert(id);
                    frontier.push(position);

                    break 'next_position;
                }
            }
        }

        tracing::debug!(?frontier, "assembled initial frontier");

        'filling: loop {
            // INVARIANT: we must ensure that in each iteration of the loop, either:
            //
            // * we completely exhaust the input amount, or
            // * we completely exhaust the reserves of one of the active positions.

            // Phase 1 (Sensing): determine the index of the constraining position by
            // executing along the frontier, tracking which hops are
            // constraining.

            let mut constraining_index = None;
            let mut current_input = Value {
                amount: input.amount,
                asset_id: input.asset_id,
            };
            tracing::debug!(current_input = ?current_input, "sensing frontier capacity with test amount");

            for (i, position) in frontier.iter().enumerate() {
                let (unfilled, new_reserves, output) = position
                    .phi
                    .fill(current_input, &position.reserves)
                    .expect("asset ids should match");

                if unfilled.amount > Amount::zero() {
                    tracing::debug!(
                        i,
                        current_input = ?current_input.amount,
                        unfilled = ?unfilled.amount,
                        output = ?output.amount,
                        old_reserves = ?position.reserves,
                        new_reserves = ?new_reserves,
                        "could not completely fill input amount, marking as constraining"
                    );
                    // We found a pair that constrains how much we can fill along this frontier.
                    constraining_index = Some(i);
                } else {
                    tracing::debug!(
                        i,
                        current_input = ?current_input.amount,
                        unfilled = ?unfilled.amount,
                        output = ?output.amount,
                        old_reserves = ?position.reserves,
                        new_reserves = ?new_reserves,
                        "completely filled "
                    );
                }

                current_input = output;
            }

            // Phase 2 (Filling): execute along the path, using the constraint information we sensed above.

            // Special case:
            // If there was no constraining index, we can completely fill the input amount,
            // upholding the invariant for loop exits by exhausting the input amount.
            let Some(constraining_index) = constraining_index else {
                tracing::debug!("no constraining index found, completely filling input amount");

                // Because there's no constraint, we can completely fill the entire input amount.
                let current_input = input;
                trace[0].amount += current_input.amount;

                let mut current_value = current_input;
                for (i, position) in frontier.iter_mut().enumerate() {
                    let (unfilled, new_reserves, output) = position
                        .phi
                        .fill(current_value, &position.reserves)
                        .expect("asset ids should match");

                    assert_eq!(
                        unfilled.amount,
                        Amount::zero(),
                        "unfilled amount for unconstrained frontier should be zero"
                    );

                    position.reserves = new_reserves;
                    current_value = output;
                    trace[i + 1].amount += output.amount;
                }
                let current_output = current_value;

                // Update the input and output amounts tracked outside of the loop:
                input.amount = input.amount - current_input.amount;
                output.amount = output.amount + current_output.amount;

                tracing::debug!(
                    current_input = ?current_input.amount,
                    current_output = ?current_output.amount,
                    input = ?input.amount,
                    output = ?output.amount,
                    "filled input amount completely, breaking loop"
                );

                break 'filling;
            };

            // If there was a constraining position along the path, we want to
            // completely consume its reserves, then work "outwards" along the
            // path, propagating rounding errors forwards to the end of the path
            // and backwards to the input.

            // Example:
            // 0     1     2     3      4         [trace index]
            // UM => GM => GN => USD => ETH       [asset id]
            //     0     1     2      3           [pair index]
            //
            // Suppose that pair 2 is the constraining pair, with 0.1 USD
            // reserves.  To completely consume the 0.1 USD reserves, we need
            // work backwards along the path to compute a sequence of input
            // amounts that are valid trades to get to 0.1 USD output at pair 2,
            // and work forwards to compute the corresponding output amounts at
            // the end of the path.

            // We want to completely consume the 0.1 USD reserves, then work backwards
            // along the path to compute input amounts that are valid trades to get

            let exactly_consumed_reserves = Value {
                amount: frontier[constraining_index]
                    .reserves_for(pairs[constraining_index].end)
                    .expect("asset ids should match"),
                asset_id: pairs[constraining_index].end,
            };

            tracing::debug!(
                constraining_index,
                exactly_consumed_reserves = ?exactly_consumed_reserves.amount,
                "found constraining index, attempting to completely consume reserves"
            );

            // Work backwards along the path from the constraining position.
            let mut current_value = exactly_consumed_reserves;
            for i in (0..=constraining_index).rev() {
                trace[i + 1].amount += current_value.amount;

                let (new_reserves, prev_input) = frontier[i]
                    .phi
                    .fill_output(&frontier[i].reserves, current_value)
                    .expect("asset ids should match")
                    .expect("working backwards from most-constraining position should not exceed reserves");

                tracing::debug!(
                    i,
                    current_value = ?current_value.amount,
                    prev_input = ?prev_input.amount,
                    old_reserves = ?frontier[i].reserves,
                    new_reserves = ?new_reserves,
                    "found previous input for current value"
                );

                frontier[i].reserves = new_reserves;
                current_value = prev_input;
            }
            trace[0].amount += current_value.amount;

            // Now we know the amount of input we need to supply to exactly
            // consume the reserves of the constraining position, and have
            // updated reserves for all positions up to and including the
            // constraining one.
            let current_input = current_value;

            // Finally, we work forwards along the path from the constraining
            // position to the output:
            let mut current_value = exactly_consumed_reserves;
            for i in (constraining_index + 1)..frontier.len() {
                let (unfilled, new_reserves, output) = frontier[i]
                    .phi
                    .fill(current_value, &frontier[i].reserves)
                    .expect("asset ids should match");

                assert_eq!(
                    unfilled.amount,
                    Amount::zero(),
                    "unfilled amount for unconstrained frontier should be zero"
                );

                frontier[i].reserves = new_reserves;
                current_value = output;
                trace[i + 1].amount += output.amount;
            }
            // Now we know the amount of output we can supply, and have updated
            // reserves for all positions on the frontier.
            let current_output = current_value;

            // Phase 3: update the frontier and prepare for the next iteration.

            // Update the input and output amounts tracked outside of the loop:
            input.amount = input.amount - current_input.amount;
            output.amount = output.amount + current_output.amount;

            tracing::debug!(
                current_input = ?current_input.amount,
                current_output = ?current_output.amount,
                input = ?input.amount,
                output = ?output.amount,
                "completed fill iteration, updating frontier"
            );

            // We're going to discard the constraining position we just consumed,
            // so write its updated reserves before we replace it on the frontier.
            // The other positions will be written out either when they're fully
            // consumed, or when we finish filling.
            self.put_position(frontier[constraining_index].clone());

            let current_effective_price =
                U128x128::from(current_input.amount) / U128x128::from(current_output.amount);

            // Check whether we should stop filling because we've exceeded the spill price:
            // TODO: is comparing Options the right behavior here? can we
            // continue rolling over dust positions without allowing arbitrarily
            // higher prices than the requested spill price?
            if spill_price.is_some() && current_effective_price > spill_price {
                tracing::debug!(
                    // HACK: render infinite price as 0
                    spill_price = %spill_price.unwrap_or_default(),
                    current_effective_price = %current_effective_price.unwrap_or_default(),
                    "exceeded spill price, breaking loop"
                );
                break 'filling;
            }

            // Find the next position to replace the one we just consumed.
            'next_position: loop {
                let next_position_id = match positions_by_price
                    .get_mut(&pairs[constraining_index])
                    .unwrap()
                    .as_mut()
                    .next()
                    .await
                    .transpose()?
                {
                    // If none is available, we can't keep filling, and need to break.
                    None => {
                        tracing::debug!(
                            pair = ?pairs[constraining_index],
                            "no more positions available for pair, breaking loop"
                        );
                        break 'filling;
                    }
                    // Otherwise, we need to check that the position is not already
                    // part of the current frontier.
                    Some(position_id) if !frontier_position_ids.contains(&position_id) => {
                        position_id
                    }
                    // Otherwise, continue to the next position in the stream.
                    Some(position_id) => {
                        tracing::debug!(?position_id, "skipping position already in frontier");
                        continue 'next_position;
                    }
                };

                let next_position = self
                    .position_by_id(&next_position_id)
                    .await?
                    .expect("indexed position should exist");

                tracing::debug!(
                    constraining_index,
                    ?next_position_id,
                    ?next_position,
                    "replacing constraining position in frontier",
                );

                frontier_position_ids.insert(next_position_id);
                frontier[constraining_index] = next_position;
                // TODO: should we remove the old position's ID from the ID set?

                // Having updated the frontier, it's time to break out of the loop.
                break 'next_position;
            }
        }

        // We need to save these positions, because we mutated their state, even
        // if we didn't fully consume their reserves.
        for position in frontier {
            self.put_position(position);
        }
        // Add the trace to the object store:
        tracing::debug!(?trace, "recording trace of filled route");
        let mut swap_execution: im::Vector<Vec<Value>> =
            self.object_get("swap_execution").unwrap_or_default();
        swap_execution.push_back(trace);
        self.object_put("swap_execution", swap_execution);

        // cleanup / finalization
        Ok((input, output))
    }
}

impl<S: StateWrite + ?Sized> FillRoute2 for S {}

/// Breaksdown a route into a collection of `DirectedTradingPair`, this is mostly useful
/// for debugging right now.
fn breakdown_route(route: &[asset::Id]) -> Result<Vec<DirectedTradingPair>> {
    if route.len() < 2 {
        Err(anyhow!("route length must be >= 2"))
    } else {
        let mut pairs = vec![];
        for pair in route.windows(2) {
            let start = pair[0].clone();
            let end = pair[1].clone();
            pairs.push(DirectedTradingPair::new(start, end));
        }
        Ok(pairs)
    }
}
