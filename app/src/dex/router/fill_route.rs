use std::collections::BTreeMap;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{lp::position, DirectedTradingPair},
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_storage::{StateDelta, StateWrite};
use tracing::debug;

use crate::dex::{PositionManager, PositionRead};

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
    async fn fill_route(
        &mut self,
        mut input: Value,
        route: &[asset::Id],
        spill_price: U128x128,
    ) -> Result<(Value, Value)> {
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
            let mut tmp_state = StateDelta::new(&self);
            let mut constraining_index = None;
            let mut constraining_position = None;
            let mut cur_value = input.clone();
            let mut effective_price = U128x128::from(1u64);
            for (i, next_asset) in route.iter().enumerate() {
                let Some(position) = tmp_state
                    .best_position(&DirectedTradingPair {
                        start: cur_value.asset_id,
                        end: *next_asset,
                    })
                    .await? else {
                        // If there's no positions left on the route, we can't keep filling.
                        break 'filling;
                    };

                let position_price = position
                    .phi
                    .orient_end(*next_asset)
                    .unwrap()
                    .effective_price();
                effective_price = (effective_price * position_price).unwrap();

                let (unfilled, output) = tmp_state.fill_against(input, &position.id()).await?;

                // If there's an unfilled input, that means we were constrained on this leg of the path.
                if unfilled.amount > 0u64.into() {
                    constraining_index = Some(i);
                    constraining_position = Some(position);
                }
                cur_value = output;
            }
            // Now that we found the capacity constraint, drop the tmp_state,
            // so we don't accidentally use its mutated state for execution.
            std::mem::drop(tmp_state);

            // If the effective price exceeds the spill price, stop filling.
            if effective_price > spill_price {
                break 'filling;
            }

            // Now `constraining_index` tells us which leg of the path was
            // constraining.  We want to ensure that we use its entire reserves,
            // not leaving any dust, so that we continue making forward
            // progress.
            let input_capacity = match (constraining_index, constraining_position) {
                (Some(index), Some(position)) => {
                    // can we use effective_price?
                    // todo: do we need to save a vec of prices for each hop to
                    // be able to work backwards, rounding up each time?
                    // how do we simultaneously guarantee that:
                    // - we consume the constraining position's reserves exactly
                    // - we never exceed any other position's reserves when rounding up
                    todo!("work backwards from reserves_of(route[constraining_index])")
                }
                (None, None) => {
                    // There's no capacity constraint, we can execute the entire input.
                    input.amount
                }
                _ => unreachable!("index and position are set together"),
            };

            // Now execute along the path on the actual state
            let mut cur_value = Value {
                amount: input_capacity,
                asset_id: input.asset_id,
            };

            for next_asset in route.iter() {
                let position = self
                    .best_position(&DirectedTradingPair {
                        start: cur_value.asset_id,
                        end: *next_asset,
                    })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("unexpectedly missing position"))?;

                let (unfilled, output) = self.fill_against(input, &position.id()).await?;
                // If there's an unfilled input, that means we were constrained on this leg of the path.
                if unfilled.amount > 0u64.into() {
                    return Err(anyhow::anyhow!(
                        "internal error: unfilled amount after filling against {:?}",
                        position.id(),
                    ));
                }
                cur_value = output;
            }

            if cur_value.amount == 0u64.into() {
                // TODO can this every happen?
                break 'filling;
            }

            // Now record the input we consumed and the output we gained:
            input.amount = input.amount - input_capacity;
            output.amount = output.amount + cur_value.amount;
        }

        Ok((input, output))
    }

    // to keep the main logic decluttered, this shouldn't be part of the final production
    /// Returns a tuple containing:
    ///     - an order list of constraining positions
    ///     - the effective price along the route.
    async fn find_constraints(
        &mut self,
        input: Value,
        route: &[asset::Id],
    ) -> Result<(Vec<(usize, position::Position)>, U128x128)> {
        let mut tmp_state = StateDelta::new(&self);
        let mut constraints: Vec<(usize, position::Position)> = vec![];
        let mut current_input = input.clone();
        let mut effective_price = U128x128::from(1u64);
        for (i, next_asset) in route.iter().enumerate().skip(1) {
            println!("{i} with {next_asset:?}");
            let Some(position) = tmp_state
                .best_position(&DirectedTradingPair {
                    start: current_input.asset_id,
                    end: *next_asset,
                })
                .await? else {
                    println!("no positions found for pair: {:?} -> {:?}", current_input.asset_id, *next_asset);
                    panic!("no position found!");

                    break;
                };

            let position_price = position
                .phi
                .orient_end(*next_asset)
                .unwrap()
                .effective_price();

            // Record (and ignore, for now) the effective price along the path.
            effective_price = (effective_price * position_price).unwrap();

            let (unfilled, output) = tmp_state
                .fill_against(current_input, &position.id())
                .await?;

            // We have found a hop in the path that bottlenecks execution.
            if unfilled.amount > 0u64.into() {
                println!("###################################################");
                println!("constraint at {i} for position: {position:?}");
                println!("trying to fill {current_input:?}");
                println!("lambda_1: {unfilled:?}");
                println!("lambda_2: {output:?}");
                println!("with effective_price {effective_price}");
                println!("vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv");
                constraints.push((i, position))
            } else {
                println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
                println!("perfect fill at {i} for position: {position:?}");
                println!("filled........ {current_input:?}");
                println!("lambda_1: {unfilled:?}");
                println!("lambda_2: {output:?}");
                println!("with effective_price {effective_price}");
                println!("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&");
            }
            current_input = output;
        }
        Ok((constraints, effective_price))
    }

    /// Breaksdown a route into a collection of `DirectedTradingPair`, this is mostly useful
    /// for debugging right now.
    fn breakdown_route(&self, route: &[asset::Id]) -> Vec<DirectedTradingPair> {
        // debug snip
        let mut mini_registry: BTreeMap<asset::Id, &'static str> = BTreeMap::new();

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        let penumbra = asset::REGISTRY.parse_unit("penumbra");
        let pusd = asset::REGISTRY.parse_unit("pusd");

        mini_registry.insert(gm.id(), "gm");
        mini_registry.insert(gn.id(), "gn");
        mini_registry.insert(penumbra.id(), "penumbra");
        mini_registry.insert(pusd.id(), "pusd");
        // snip

        let mut pairs = vec![];
        println!("route:");
        for i in 0..(route.len() - 1) {
            let start = route[i];
            let end = route[i + 1];
            let pair = DirectedTradingPair::new(start, end);
            println!("  {i}: {}-{}", mini_registry[&start], mini_registry[&end]);
            pairs.push(pair);
        }
        pairs
    }

    // erwan fork of fill_route
    async fn fill_route2(
        &mut self,
        mut input: Value,
        route: &[asset::Id],
        spill_price: U128x128,
    ) -> Result<(Value, Value)> {
        let source = route[0];
        let target = route[route.len() - 1];
        let total_pair = DirectedTradingPair::new(source, target);

        // actual amount filled
        let total_delta_1_star = Value {
            asset_id: source,
            amount: Amount::zero(),
        };
        let total_lambda_1_star = total_delta_1_star.clone();
        let total_lambda_2_star = Value {
            asset_id: target,
            amount: Amount::zero(),
        };

        // Breakdown the route into a sequence of pairs to visit.
        let mut pairs = self.breakdown_route(route);

        let mut output = Value {
            amount: 0u64.into(),
            asset_id: route
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("called fill_route with empty route"))?,
        };

        'filling: while input.amount > 0u64.into() {
            println!("input.amount={:?}", input.amount);

            println!("finding constraints.");
            // First, try to determine the capacity at the current price,
            // by simulating execution of the max amount on an ephemeral state fork.
            // Writing the results to the new StateDelta ensures that if the path has a cycle,
            // we'll see our own execution changes later in the path.
            let (constraints, effective_price) = self.find_constraints(input, route).await?;

            println!("constraints found!");

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
                    let r1 = constraining_position.reserves.r1;
                    let r2 = constraining_position.reserves.r2;

                    // can we use effective_price?
                    // todo: do we need to save a vec of prices for each hop to
                    // be able to work backwards, rounding up each time?
                    // how do we simultaneously guarantee that:
                    // - we consume the constraining position's reserves exactly
                    // - we never exceed any other position's reserves when rounding up
                    todo!("work backwards from reserves_of(route[constraining_index])")
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
            for next_asset in route.iter() {
                let position = self
                    .best_position(&DirectedTradingPair {
                        start: current_value.asset_id,
                        end: *next_asset,
                    })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("unexpectedly missing position"))?;

                let (unfilled, output) = self.fill_against(input, &position.id()).await?;
                // If there's an unfilled input, that means we were constrained on this leg of the path.
                if unfilled.amount > 0u64.into() {
                    return Err(anyhow::anyhow!(
                        "internal error: unfilled amount after filling against {:?}",
                        position.id(),
                    ));
                }
                current_value = output;
            }

            if current_value.amount == 0u64.into() {
                // TODO can this every happen?
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
