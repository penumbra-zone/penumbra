use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{asset, dex::DirectedTradingPair, fixpoint::U128x128, Value};
use penumbra_storage::{StateDelta, StateWrite};

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
}
