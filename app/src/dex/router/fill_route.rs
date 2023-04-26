use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{lp::position, DirectedTradingPair},
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_storage::{StateDelta, StateWrite};

use crate::dex::{PositionManager, PositionRead};

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
    /// Finds the constraining hops for a given route and input,
    /// and returns a tuple consisting of:
    ///     - an ordered list of `Position` and their respective saturating input
    ///     - the best `Position` for each hop of the route
    async fn find_constraints(
        &mut self,
        input: Value,
        route: &[asset::Id],
    ) -> Result<(Vec<(position::Position, Amount)>, Vec<position::Position>)> {
        let mut tmp_state = StateDelta::new(&self);
        let mut current_input = input.clone();

        let mut constraining_positions: Vec<(position::Position, Amount)> = vec![];
        let mut best_positions: Vec<position::Position> = vec![];
        let mut accumulated_effective_price = U128x128::from(1u64);

        for (_i, next_asset) in route.iter().enumerate().skip(1) {
            let Some(position) = tmp_state
                .best_position(&DirectedTradingPair {
                    start: current_input.asset_id,
                    end: *next_asset,
                })
                .await? else {
                    return Err(anyhow!("exhausted positions on hop {}-{}", current_input.asset_id, *next_asset))
                };

            best_positions.push(position.clone());

            let (unfilled, output) = tmp_state
                .fill_against(current_input, &position.id())
                .await?;

            let position_price = position
                .phi
                .orient_end(*next_asset)
                .unwrap()
                .effective_price();

            accumulated_effective_price =
                (accumulated_effective_price * position_price).expect("TODO(erwan): write proof");

            // We have found a hop in the path that bottlenecks execution.
            if unfilled.amount > Amount::zero() {
                let lambda_2 = position
                    .reserves_for(*next_asset)
                    .expect("the position has reserves for its numeraire");

                let delta_1_star = (U128x128::from(lambda_2) * accumulated_effective_price)
                    .expect("TODO(erwan): write proof");

                let saturating_input: Amount = delta_1_star.round_up().try_into()?;

                constraining_positions.push((position, saturating_input));
            }

            current_input = output;
        }

        Ok((constraining_positions, best_positions))
    }

    /// Breaksdown a route into a collection of `DirectedTradingPair`, this is mostly useful
    /// for debugging right now.
    fn breakdown_route(&self, route: &[asset::Id]) -> Result<Vec<DirectedTradingPair>> {
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

    async fn fill_route(
        &mut self,
        mut input: Value,
        route: &[asset::Id],
        spill_price: U128x128,
    ) -> Result<(Value, Value)> {
        // Breakdown the route into a sequence of pairs to visit.
        let pairs = self.breakdown_route(route)?;

        let mut output = Value {
            amount: 0u64.into(),
            asset_id: route
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("called fill_route with empty route"))?,
        };

        'filling: while input.amount > 0u64.into() {
            // Our method is based on the assurance provided by the routing algorithm: there exist a route with
            // a positive capacity and an effective price that's similar or better than the specified `spill_price`.
            // The role of the fill-phase is to try maximize the amount of flow up to the `spill_price`.
            // We naively try to route as much input as possible on the first pass to identify constraining hops.
            // For every constraint, we find the correspond input capacity for which they are "saturated".
            let (constraining_hops, best_positions) = self.find_constraints(input, route).await?;
            let effective_price = best_positions.clone().into_iter().zip(pairs.clone()).fold(
                U128x128::from(1u64),
                |acc, (pos, pair)| {
                    (acc * pos.phi.orient_end(pair.end).unwrap().effective_price()).unwrap()
                },
            );

            tracing::debug!(?effective_price, "effective price across the route");
            tracing::debug!(num = constraining_hops.len(), "found constraints");

            // If the effective price exceeds the spill price, stop filling.
            // TODO(erwan): having a measure of marginal price per capacity (or depth) over the route
            //              should be useful here. For example, we could hit the `spill_priover ce` trying to
            //              route `input` on the route but it might be possible to route some inventory
            //              that's smaller than `input`.
            if effective_price > spill_price {
                tracing::debug!(?effective_price, ?spill_price, "spill price hit!");
                break 'filling;
            }

            let input_capacity = match constraining_hops.last() {
                Some((_constraining_position, saturating_input)) => {
                    // It is not sufficient to pick the last constrait and lift it, because an earlier constraint in the route may be
                    // more restraining. Instead, we must identity the largest "saturating input" that we can push through the route such
                    // that we are able to lift the most limiting constraint.
                    // TODO(erwan): should fold this into a `find_and_solve` routine.
                    let min_delta_1_star = constraining_hops.iter().fold(
                        saturating_input.clone(),
                        |current_min, (_, saturating_input)| {
                            Amount::min(current_min, saturating_input.clone())
                        },
                    );

                    min_delta_1_star
                }
                None => {
                    // There's no capacity constraint, we can execute the entire input.
                    input.amount
                }
            };

            let mut current_value = Value {
                amount: input_capacity,
                asset_id: input.asset_id,
            };

            // Now that we know `delta_1_star`, we can execute along the route,
            // knowing that the ultimate constraint has been lifted.
            for next_asset in route.iter().skip(1) {
                let position = self
                    .best_position(&DirectedTradingPair {
                        start: current_value.asset_id,
                        end: *next_asset,
                    })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("unexpectedly missing position"))?;
                let (unfilled, output) = self.fill_against(current_value, &position.id()).await?;

                // If there's an unfilled input, that means we were constrained on this leg of the path.
                if unfilled.amount > 0u64.into() {
                    tracing::error!(
                        ?unfilled,
                        ?position,
                        ?current_value,
                        "residual unfilled amount here"
                    );
                    return Err(anyhow::anyhow!(
                        "internal error: unfilled amount after filling against {:?}",
                        position.id(),
                    ));
                }
                current_value = output;
            }

            if current_value.amount == 0u64.into() {
                println!("zero current value.");
                // Note: this can be hit during dust fills
                // TODO(erwan): craft `test_dust_fill_zero_value` to prove this.
                panic!("zero current value");
                break 'filling;
            }

            // Now record the input we consumed and the output we gained:
            input.amount = input.amount - input_capacity;
            output.amount = output.amount + current_value.amount;
        }

        Ok((input, output))
    }
}

impl<T: PositionManager> FillRoute for T {}
