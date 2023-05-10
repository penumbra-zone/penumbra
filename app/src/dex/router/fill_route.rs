use std::{
    collections::{BTreeSet, HashMap},
    pin::Pin,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use penumbra_crypto::{
    asset,
    dex::{
        lp::position::{self, Position},
        DirectedTradingPair,
    },
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_storage::{StateDelta, StateRead, StateWrite};
use tracing::instrument;

use crate::dex::{PositionManager, PositionRead};

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
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
        // Build a transaction for this execution, so if we error out at any
        // point we don't leave the state in an inconsistent state.  This is
        // particularly important for this method, because we lift position data
        // out of the state and modify it in-memory, writing it only as we fully
        // consume positions.
        let mut this = StateDelta::new(self);

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
                .or_insert_with(|| this.positions_by_price(&pair));
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

        let mut frontier = Frontier::load(&this, &mut positions_by_price, &pairs).await?;
        tracing::debug!(?frontier, "assembled initial frontier");

        'filling: loop {
            // INVARIANT: we must ensure that in each iteration of the loop, either:
            //
            // * we completely exhaust the input amount, or
            // * we completely exhaust the reserves of one of the active positions.

            // Phase 1 (Sensing): determine the index of the constraining position by
            // executing along the frontier, tracking which hops are
            // constraining.
            let constraining_index = frontier.sense_capacity_constraint(input);

            // Phase 2 (Filling): execute along the path, using the constraint information we sensed above.

            // Special case:
            // If there was no constraining index, we can completely fill the input amount,
            // upholding the invariant for loop exits by exhausting the input amount.
            let Some(constraining_index) = constraining_index else {
                tracing::debug!("no constraining index found, completely filling input amount");

                // Because there's no constraint, we can completely fill the entire input amount.
                let current_input = input;
                // We have to manually update the trace here, because fill_forward
                // doesn't handle the input amount, only things that come after it.
                trace[0].amount += current_input.amount;
                let current_output = frontier.fill_forward(0, input, &mut trace);

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
                amount: frontier.positions[constraining_index]
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
            let current_input =
                frontier.fill_backward(constraining_index, exactly_consumed_reserves, &mut trace);

            // Now we know the amount of input we need to supply to exactly
            // consume the reserves of the constraining position, and have
            // updated reserves for all positions up to and including the
            // constraining one.

            // Finally, we work forwards along the path from the constraining
            // position to the output:
            let current_output = frontier.fill_forward(
                constraining_index + 1,
                exactly_consumed_reserves,
                &mut trace,
            );
            // Now we know the amount of output we can supply, and have updated
            // reserves for all positions on the frontier.

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

            // Try to find a new position to replace the one we just filled.
            if !frontier
                .replace_position(
                    constraining_index,
                    &pairs,
                    &mut this,
                    &mut positions_by_price,
                )
                .await?
            {
                tracing::debug!("no more positions to replace, breaking loop");
                break 'filling;
            };

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
        }

        // We need to save these positions, because we mutated their state, even
        // if we didn't fully consume their reserves.
        frontier.save(&mut this);

        // Add the trace to the object store:
        tracing::debug!(?trace, "recording trace of filled route");
        let mut swap_execution: im::Vector<Vec<Value>> =
            this.object_get("swap_execution").unwrap_or_default();
        swap_execution.push_back(trace);
        this.object_put("swap_execution", swap_execution);

        // Apply the state transaction now that we've reached the end without errors.
        this.apply();

        // cleanup / finalization
        Ok((input, output))
    }
}

impl<S: StateWrite> FillRoute for S {}

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

type PositionsByPrice =
    HashMap<DirectedTradingPair, Pin<Box<dyn Stream<Item = Result<position::Id>> + Send>>>;

/// A frontier of least-priced positions along a route.
#[derive(Debug)]
struct Frontier {
    /// A list of the positions on the route.
    pub positions: Vec<Position>,
    /// A set of position IDs of positions contained in the frontier.
    ///
    /// This lets us correctly handle the case where we traverse the same macro-edge
    /// in opposite directions, and a position has nonzero reserves of both assets
    /// and shows up in both position streams (even though we must only use it once).
    pub position_ids: BTreeSet<position::Id>,
}

impl Frontier {
    async fn load(
        state: impl StateRead,
        positions_by_price: &mut PositionsByPrice,
        pairs: &[DirectedTradingPair],
    ) -> Result<Frontier> {
        let mut positions = Vec::new();
        let mut position_ids = BTreeSet::new();

        for pair in pairs {
            'next_position: loop {
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
                if !position_ids.contains(&id) {
                    // TODO: fold positions into position_by_id stream
                    // so separate state lookup not necessary
                    let position = state
                        .position_by_id(&id)
                        .await?
                        .ok_or_else(|| anyhow!("position with indexed id {:?} not found", id))?;

                    position_ids.insert(id);
                    positions.push(position);

                    break 'next_position;
                }
            }
        }

        Ok(Frontier {
            positions,
            position_ids,
        })
    }

    fn save(&self, mut state: impl StateWrite) {
        for position in &self.positions {
            state.put_position(position.clone());
        }
    }

    // Returns Ok(true) if a new position was found to replace the given one,
    // or Ok(false) if there are no more positions available for the given pair.
    #[instrument(skip(self, pairs, state, positions_by_price))]
    async fn replace_position(
        &mut self,
        index: usize,
        pairs: &[DirectedTradingPair],
        mut state: impl StateWrite,
        positions_by_price: &mut PositionsByPrice,
    ) -> Result<bool> {
        let replaced_position_id = self.positions[index].id();
        tracing::debug!(?replaced_position_id, "replacing position");

        // First, save the position we're about to replace.  We're going to
        // discard it, so write its updated reserves before we replace it on the
        // frontier.  The other positions will be written out either when
        // they're fully consumed, or when we finish filling.
        state.put_position(self.positions[index].clone());

        loop {
            let pair = &pairs[index];
            let next_position_id = match positions_by_price
                .get_mut(pair)
                .unwrap()
                .as_mut()
                .next()
                .await
                .transpose()?
            {
                // If none is available, we can't keep filling, and need to signal as such.
                None => {
                    tracing::debug!(?pair, "no more positions available for pair");
                    return Ok(false);
                }
                // Otherwise, we need to check that the position is not already
                // part of the current frontier.
                Some(position_id) if !self.position_ids.contains(&position_id) => position_id,
                // Otherwise, continue to the next position in the stream.
                Some(position_id) => {
                    tracing::debug!(?position_id, "skipping position already in frontier");
                    continue;
                }
            };

            let next_position = state
                .position_by_id(&next_position_id)
                .await?
                .expect("indexed position should exist");

            tracing::debug!(
                ?next_position_id,
                ?next_position,
                "replacing constraining position in frontier",
            );

            self.position_ids.remove(&replaced_position_id);
            self.position_ids.insert(next_position_id);
            self.positions[index] = next_position;

            return Ok(true);
        }
    }

    /// Senses which, if any, position along the frontier is a capacity
    /// constraint for the given input amount.
    #[instrument(skip(self, input), fields(input = ?input.amount))]
    fn sense_capacity_constraint(&self, input: Value) -> Option<usize> {
        tracing::debug!("sensing frontier capacity with test amount");
        let mut constraining_index = None;
        let mut current_input = input;

        for (i, position) in self.positions.iter().enumerate() {
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

        constraining_index
    }

    #[instrument(skip(self, input, trace), fields(input = ?input.amount))]
    fn fill_forward(&mut self, start_index: usize, input: Value, trace: &mut Vec<Value>) -> Value {
        tracing::debug!("filling forward along frontier");
        let mut current_value = input;

        for i in start_index..self.positions.len() {
            let (unfilled, new_reserves, output) = self.positions[i]
                .phi
                .fill(current_value, &self.positions[i].reserves)
                .expect("asset ids should match");

            assert_eq!(
                unfilled.amount,
                Amount::zero(),
                "unfilled amount for unconstrained frontier should be zero"
            );

            self.positions[i].reserves = new_reserves;
            current_value = output;
            trace[i + 1].amount += output.amount;
        }

        current_value
    }

    #[instrument(skip(self, output, trace), fields(output = ?output.amount))]
    fn fill_backward(
        &mut self,
        start_index: usize,
        output: Value,
        trace: &mut Vec<Value>,
    ) -> Value {
        tracing::debug!("filling backward along frontier");
        let mut current_value = output;
        for i in (0..=start_index).rev() {
            trace[i + 1].amount += current_value.amount;

            let (new_reserves, prev_input) = self.positions[i]
                .phi
                .fill_output(&self.positions[i].reserves, current_value)
                .expect("asset ids should match")
                .expect(
                    "working backwards from most-constraining position should not exceed reserves",
                );

            tracing::debug!(
                i,
                current_value = ?current_value.amount,
                prev_input = ?prev_input.amount,
                old_reserves = ?self.positions[i].reserves,
                new_reserves = ?new_reserves,
                "found previous input for current value"
            );

            self.positions[i].reserves = new_reserves;
            current_value = prev_input;
        }
        trace[0].amount += current_value.amount;

        current_value
    }
}
