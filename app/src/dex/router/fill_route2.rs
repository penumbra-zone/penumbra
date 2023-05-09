use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
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

        // TODO: this does not allow traversing the same macro-edge in opposite directions
        // (This impl will produce WRONG RESULTS if you try to do that!)
        // later: implement fast path for non-opposite directions, slow path (check for duplicate position ids)

        let mut frontier = Vec::new();
        for pair in &pairs {
            let id = positions_by_price
                .get_mut(pair)
                .unwrap()
                .as_mut()
                .next()
                .await
                .ok_or_else(|| anyhow!("no positions available for pair on path {:?}", pair))??;
            // TODO: save id alongside position for deduplication ?
            let position = self
                .position_by_id(&id)
                .await?
                .ok_or_else(|| anyhow!("position with indexed id {:?} not found", id))?;
            frontier.push(position);
        }

        'filling: loop {
            // INVARIANT: we must ensure that in each iteration of the loop, either:
            //
            // * we completely exhaust the input amount, or
            // * we completely exhaust the reserves of one of the active positions.

            // Phase 1: determine the index of the constraining position by
            // executing along the frontier, tracking which hops are
            // constraining.

            let mut constraining_index = None;
            let mut current_input = Value {
                amount: std::cmp::min(
                    frontier[0]
                        .reserves_for(input.asset_id)
                        .expect("pair is for input asset"),
                    input.amount,
                ),
                asset_id: input.asset_id,
            };

            for (i, position) in frontier.iter().enumerate() {
                let (unfilled, _new_reserves, output) = position
                    .phi
                    .fill(current_input, &position.reserves)
                    .expect("asset ids should match");

                if unfilled.amount > Amount::zero() {
                    // We found a pair that constrains how much we can fill along this frontier.
                    constraining_index = Some(i);
                }

                current_input = output;
            }

            // Phase 2: execute along the path, using the constraint information we sensed above.
            let Some(constraining_index) = constraining_index else {
                // If there was no constraining index, we can completely fill the input amount,
                // upholding the invariant for loop exits.
                tracing::debug!("no constraining index found, completely filling input amount");

                let mut current_input = input;
                trace[0].amount += current_input.amount;

                for (i, position) in frontier.iter_mut().enumerate() {
                    let (unfilled, new_reserves, output) = position
                        .phi
                        .fill(current_input, &position.reserves)
                        .expect("asset ids should match");

                    assert_eq!(unfilled.amount, Amount::zero(), "unfilled amount for unconstrained frontier should be zero");

                    position.reserves = new_reserves;

                    current_input = output;
                    trace[i + 1].amount += output.amount;
                }

                break 'filling;
            };

            // If there was a constraining position along the path, we want to
            // completely consume its reserves, then work "outwards" along the
            // path, propagating rounding errors forwarsd to the end of the path
            // and backwards to the input.
            let exactly_consumed_reserves = frontier[constraining_index]
                .reserves_for(pairs[constraining_index].end)
                .expect("asset ids should match");

            // Work backwards along the path from the constraining position.
            let mut implied_input = Value {
                amount: exactly_consumed_reserves,
                asset_id: pairs[constraining_index].end,
            };
            for i in (0..=constraining_index).rev() {}

            // Work forwards along the path from the constraining position.
            todo!();

            // Replace the consumed position on the frontier:
            self.put_position(frontier[constraining_index].clone());

            // replace the consumed position with the next item from the positions_by_price streams
        }

        // We need to save these positions, because mutated their state, even if we didn't fully consume them.
        for position in frontier {
            self.put_position(position);
        }

        // cleanup / finalization

        todo!()
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
