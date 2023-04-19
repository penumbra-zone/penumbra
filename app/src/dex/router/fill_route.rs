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
use sha2::digest::consts::U1;
use tracing::debug;

use crate::dex::{PositionManager, PositionRead};
use futures::StreamExt;

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
    // to keep the main logic decluttered, this shouldn't be part of the final production
    /// Returns a tuple containing:
    ///     - an order list of constraining positions
    ///     - the effective price along the route.
    async fn find_constraints(
        &mut self,
        input: Value,
        route: &[asset::Id],
    ) -> Result<(Vec<(usize, position::Position)>, Vec<position::Position>)> {
        let mut tmp_state = StateDelta::new(&self);
        let mut constraints: Vec<(usize, position::Position)> = vec![];
        let mut current_input = input.clone();
        let mut effective_price = U128x128::from(1u64);
        let mut positions: Vec<position::Position> = vec![];

        for (i, next_asset) in route.iter().enumerate().skip(1) {
            let Some(position) = tmp_state
                .best_position(&DirectedTradingPair {
                    start: current_input.asset_id,
                    end: *next_asset,
                })
                .await? else {
                    break;
                };

            let position_price = position
                .phi
                .orient_end(*next_asset)
                .unwrap()
                .effective_price();

            // Record (and ignore, for now) the effective price along the path.
            effective_price = (effective_price * position_price).unwrap();
            positions.push(position.clone());

            let (unfilled, output) = tmp_state
                .fill_against(current_input, &position.id())
                .await?;

            // We have found a hop in the path that bottlenecks execution.
            if unfilled.amount > 0u64.into() {
                constraints.push((i, position))
            } else {
                // log perfect fill
            }
            current_input = output;
        }

        Ok((constraints, positions))
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
        let source = route[0];
        let target = route[route.len() - 1];
        let total_pair: DirectedTradingPair = DirectedTradingPair::new(source, target);

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
            let (constraints, positions) = self.find_constraints(input, route).await?;
            let effective_price = positions
                .clone()
                .into_iter()
                .fold(U128x128::from(1u64), |acc, pos| {
                    (acc * pos.phi.component.effective_price()).unwrap()
                });

            tracing::debug!(?effective_price, "effective price across the route");
            tracing::debug!(num = constraints.len(), "found constraints");
            let inv_effective_price = (U128x128::from(1u64) / effective_price).unwrap();

            // If the effective price exceeds the spill price, stop filling.
            if effective_price > spill_price {
                tracing::debug!(?effective_price, ?spill_price, "spill price hit.");
                break 'filling;
            }

            // Now `constraining_index` tells us which leg of the path was
            // constraining.  We want to ensure that we use its entire reserves,
            // not leaving any dust, so that we continue making forward
            // progress.
            let input_capacity = match constraints.last() {
                Some((constraining_index, constraining_position)) => {
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
                    let delta_1_star = (U128x128::from(lambda_2) * inv_effective_price).unwrap();
                    delta_1_star.round_up().try_into()?
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

            for next_asset in route.iter().skip(1) {
                let position = self
                    .best_position(&DirectedTradingPair {
                        start: current_value.asset_id,
                        end: *next_asset,
                    })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("unexpectedly missing position"))?;
                let (unfilled, output) = self.fill_against(current_value, &position.id()).await?;

                // TODO(erwan): We can't really get perfect fills because of rounding, so commenting this out for now.
                // If there's an unfilled input, that means we were constrained on this leg of the path.
                // if unfilled.amount > 0u64.into() {
                //     return Err(anyhow::anyhow!(
                //         "internal error: unfilled amount after filling against {:?}",
                //         position.id(),
                //     ));
                // }
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
