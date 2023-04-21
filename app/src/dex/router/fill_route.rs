use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{lp::position, DirectedTradingPair},
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_storage::{StateDelta, StateWrite};
use sha2::digest::consts::U12;

use crate::dex::{PositionManager, PositionRead};
use futures::StreamExt;

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
    /// Finds the constraining hops alongside a given route for a specified input amount, and
    /// returns a tuple of:
    ///         - an ordered list of `Position`s corresponding to constraining hops
    ///         - an ordered list of the best `Position`s for every hop on the route.
    async fn find_constraints(
        &mut self,
        input: Value,
        route: &[asset::Id],
    ) -> Result<(
        Vec<(usize, Amount, position::Position)>,
        Vec<position::Position>,
    )> {
        let mut tmp_state = StateDelta::new(&self);
        let mut constraining_positions: Vec<(usize, Amount, position::Position)> = vec![];
        let mut current_input = input.clone();
        let mut best_positions: Vec<position::Position> = vec![];
        let mut accumulated_effective_price = U128x128::from(1u64);

        for (i, next_asset) in route.iter().enumerate().skip(1) {
            let Some(position) = tmp_state
                .best_position(&DirectedTradingPair {
                    start: current_input.asset_id,
                    end: *next_asset,
                })
                .await? else {
                    panic!("no positions!");
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

            accumulated_effective_price = (accumulated_effective_price * position_price)
                .expect("TODO(erwan): think through why this is not overflowable.");

            // We have found a hop in the path that bottlenecks execution.
            if unfilled.amount > 0u64.into() {
                let lambda_2 = position
                    .reserves_for(*next_asset)
                    .expect("the position has reserves for its numeraire");

                let delta_1_star = (U128x128::from(lambda_2) * accumulated_effective_price)
                    .expect("TODO(erwan): write up why this cannot overflow");

                let saturating_input: Amount = delta_1_star.round_up().try_into()?;

                constraining_positions.push((i, saturating_input, position));
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
        let mut pairs = self.breakdown_route(route)?;

        let mut output = Value {
            amount: 0u64.into(),
            asset_id: route
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("called fill_route with empty route"))?,
        };

        'filling: while input.amount > 0u64.into() {
            // First, try to determine the capacity at the current price,
            // by simulating execution of the max amount on an ephemeral state fork.
            // Writing the results to the new StateDelta ensures that if the path has a cycle,
            // we'll see our own execution changes later in the path.
            let (constraining_hops, best_positions) = self.find_constraints(input, route).await?;
            let effective_price = best_positions
                .clone()
                .into_iter()
                .fold(U128x128::from(1u64), |acc, pos| {
                    (acc * pos.phi.component.effective_price()).unwrap()
                });

            tracing::debug!(?effective_price, "effective price across the route");
            tracing::debug!(num = constraining_hops.len(), "found constraints");

            // If the effective price exceeds the spill price, stop filling.
            if effective_price > spill_price {
                println!("spill price hit");
                tracing::debug!(?effective_price, ?spill_price, "spill price hit.");
                break 'filling;
            }

            // Now `constraining_index` tells us which leg of the path was
            // constraining.  We want to ensure that we use its entire reserves,
            // not leaving any dust, so that we continue making forward
            // progress.
            let input_capacity = match constraining_hops.last() {
                Some((constraining_index, saturating_input, constraining_position)) => {
                    // There are a couple things to worry about here, let's reason step-by-step:
                    //      + can constraint resolution generate constraints upstream in the path?
                    //          answer: case1: no, case2:yes
                    //              example:
                    //      S -> A -> B -> C* -> T
                    //                     ^_ C is the constraint
                    //                at this point there are two different approaches:
                    //                    -> first one would be to work out what input would exactly fill the constraining position, working backwards
                    //                        to adjust the amount of flow (strictly reducing) and the proceed forward to a filled amount total_lambda_2
                    //                         | we work backwards from C*, determining how much delta_1 would turn the constraint into a perfect fill.
                    //                         | this is equivalent to reversing the path and executing for delta_1_new = lambda_2_old
                    //                     -> the second one, is to fetch the next order in the book that would let us fill the current flow.
                    //                        There are different branches possible here:
                    //                         |   + there are not any other order in the book
                    //                         )   + there are other orders in the book:
                    //                                > there is not enough depth to fill us
                    //                                > there is enough depth:
                    //                                        * the effective_price is similar
                    //                                        * the effective_price is worse:
                    //                                            i.) we fill and get above the spill price
                    //                                            ii) we fill and we're still below the spill price.

                    let lambda_2 = constraining_position
                        .reserves_for(pairs[*constraining_index - 1].end)
                        .unwrap();

                    // What happens if there are "stacked constraints"?
                    // => We know that solving the constraint by strictly min-max'ing the flow across the path
                    // lets us stay below the spill price. But, what happens when lifting the last constraint
                    // is not sufficient i.e. there is a preceding constraint that's even more constraining.
                    // This should be optimized, but I believe the solution is to take advantage of the fact that
                    // by strictly reducing the flow we know we haven't created any _new_ constraint, therefore
                    // we can individually visit previously recorded constraints, check and solve those that remain
                    // bottlenecky.

                    // When multiple constraints are found on different hops, we have to consider the case when
                    // the last constraint is not the smallest. So we want to select the smallest upper bound on
                    // `delta_1_star` that allows every constraint to be satisfied.
                    let inv_effective_price = (U128x128::from(1u64) / effective_price).unwrap();
                    let delta_1_star = (U128x128::from(lambda_2) * inv_effective_price).unwrap();
                    let delta_1_star: Amount = delta_1_star.round_up().try_into()?;

                    let min_delta_1_star = constraining_hops.iter().fold(
                        delta_1_star,
                        |current_min, (_, saturating_input, _)| {
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

            // Now execute along the path on the actual state
            let mut current_value = Value {
                amount: input_capacity,
                asset_id: input.asset_id,
            };

            println!("delta_1_star = {current_value:?}");

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
                    tracing::warn!(?unfilled, "residual unfilled amount here");
                    println!("residual unfilled: {unfilled:?}");
                    //                    return Err(anyhow::anyhow!(
                    //                        "internal error: unfilled amount after filling against {:?}",
                    //                        position.id(),
                    //                    ));
                }
                current_value = output;
                println!("output={current_value:?}");
            }

            if current_value.amount == 0u64.into() {
                println!("zero current value.");
                // Note: this can be hit during dust fills
                // TODO(erwan): craft `test_dust_fill_zero_value` to prove this.
                // panic!("zero current value");
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
