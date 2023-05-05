use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_crypto::{
    asset,
    dex::{lp::position, DirectedTradingPair},
    fixpoint::U128x128,
    Amount, Value,
};
use penumbra_storage::{StateDelta, StateWrite};
use tracing::instrument;

use crate::dex::{PositionManager, PositionRead};

#[async_trait]
pub trait FillRoute: StateWrite + Sized {
    /// Finds the constraining hops for a given route and input,
    /// and returns a tuple consisting of:
    ///     - an ordered list of `Position` and their respective saturating input
    ///     - the best `Position` for each hop of the route
    #[instrument(skip(self, input, hops))]
    async fn find_constraints(
        &mut self,
        input: Value,
        hops: &[asset::Id],
    ) -> Result<Option<(Vec<(position::Position, Amount)>, Vec<position::Position>)>> {
        let mut route = hops.to_vec();
        route.insert(0, input.asset_id);
        let pairs = self.breakdown_route(&route)?;

        let mut tmp_state = StateDelta::new(&self);
        let mut current_input = input.clone();

        let mut constraining_positions: Vec<(position::Position, Amount)> = vec![];
        let mut best_positions: Vec<position::Position> = vec![];
        let mut accumulated_effective_price = U128x128::from(1u64);

        for pair in pairs {
            let Some(position) = tmp_state
                .best_position(&pair)
                .await? else {
                    return Ok(None);
                    // return Err(anyhow!("exhausted positions on hop {}-{}", current_input.asset_id, pair.end))
                };

            best_positions.push(position.clone());

            let (unfilled, output) = tmp_state
                .fill_against(current_input, &position.id())
                .await?;

            let position_price = position.phi.orient_end(pair.end).unwrap().effective_price();

            accumulated_effective_price =
                (accumulated_effective_price * position_price).expect("TODO(erwan): write proof");

            // We have found a hop in the path that bottlenecks execution.
            if unfilled.amount > Amount::zero() {
                let lambda_2 = position
                    .reserves_for(pair.end)
                    .expect("the position has reserves for its numeraire");

                let delta_1_star = (U128x128::from(lambda_2) * accumulated_effective_price)
                    .expect("TODO(erwan): write proof");

                // SAFETY: There are two important reasons to round up the saturating
                // input here. The first one is because of asset conservation:
                // -> The first reason has to do with "asset conservation":
                // If we rounded down delta_1, we would "leak" the error term which
                // can be catastrophic: d_1 = floor(0.1), l_2 = 2
                //                      d_1 = 0, l_2 = 2
                // Instead, we want to "burn" the complement of the error term:
                //                      d_1 = ceil(0.1), l_2 = 2
                //                      d_1 = 1, l_2 = 2
                // -> The second reason is that routing execution assumes that
                // saturating inputs are positive, and in particular, nonzero.
                // Unlike preserving the asset conservation law, this is not a
                // driving the decision of rounding up here, but any changes to
                // this computation percolates into routing execution behavior.
                // Therefore, it is necessary to make sure that this assumption
                // isn't broken without making the necessary changes in the routing
                // execution logic.
                let saturating_input: Amount = delta_1_star.round_up().try_into()?;

                constraining_positions.push((position, saturating_input));
            }

            current_input = output;
        }

        Ok(Some((constraining_positions, best_positions)))
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

    #[instrument(skip(self, input, hops, spill_price))]
    async fn fill_route(
        &mut self,
        mut input: Value,
        hops: &[asset::Id],
        spill_price: Option<U128x128>,
    ) -> Result<(Value, Value)> {
        tracing::debug!(
            ?input,
            ?hops,
            ?spill_price,
            "filling along route up to spill price"
        );

        let mut route = hops.to_vec();
        route.insert(0, input.asset_id);

        // Breakdown the route into a sequence of pairs to visit.
        let pairs = self.breakdown_route(&route)?;

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

        'filling: while input.amount > 0u64.into() {
            // Our approach is based on the assurance provided by the routing phase,
            // that there exist a route with a positive capacity, and an end-to-end
            // price that is similar or better than the `spill_price`.
            // The role of routing execution is to maximize the amount of flow up to
            // that specified `spill_price`. First, we naively try to route as much
            // input as the inventory of the best position of the first hop allows.
            // Simulating execution for that input lets us identify these nodes that
            // are limiting the flow aka. "constraints". For every such constraint,
            // there's an input capacity that maximize its output without causing an
            // overflow.
            let Some((constraining_hops, best_positions)) = self.find_constraints(input, hops).await? else {
                // If any hop along the route runs out of liquidity,
                // we cannot fill and have to break early.
                break;
            };

            let effective_price = best_positions.clone().into_iter().zip(pairs.clone()).fold(
                U128x128::from(1u64),
                |acc, (pos, pair)| {
                    (acc * pos.phi.orient_end(pair.end).unwrap().effective_price()).unwrap()
                },
            );

            tracing::debug!(%effective_price, "effective price across the route");
            tracing::debug!(num = constraining_hops.len(), "found constraints");

            //  Stop filling if the effective price exceeds the spill price.
            if spill_price.map(|s| effective_price > s).unwrap_or(false) {
                tracing::debug!(?effective_price, ?spill_price, "spill price hit!");
                break 'filling;
            }

            let input_capacity = match constraining_hops.last() {
                Some((_constraining_position, saturating_input)) => {
                    // It is not sufficient to pick the last constrait and lift it,
                    // because an earlier constraint in the route might be more
                    // limiting. Instead, we identifiy the largest saturating input
                    // that we can push through the route such that the most limiting
                    // constraint is lifted.
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

            // Store the current value in the execution trace:
            trace[0].amount += input_capacity;

            // Now, we can execute along the route knowing that the most limiting
            // constraint is lifted. This means that this specific input is exactly
            // the maximum flow for the composed positions.
            for (pair_idx, pair) in pairs.iter().enumerate() {
                assert_eq!(current_value.asset_id, pair.start);
                let position = self
                    .best_position(&DirectedTradingPair {
                        start: current_value.asset_id,
                        end: pair.end,
                    })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("unexpectedly missing position"))?;
                let (unfilled, output) = self.fill_against(current_value, &position.id()).await?;

                // This should not happen, since the `current_capacity` is the
                // saturating input for the route.
                if unfilled.amount > 0u64.into() {
                    tracing::warn!(
                        ?unfilled,
                        ?current_value,
                        ?position,
                        ?pair,
                        "burning unexpected residual unfilled amount"
                    );
                    /*
                    return Err(anyhow::anyhow!(
                        "internal error: unfilled amount after filling against {:?}",
                        position.id(),
                    ));
                    */
                }
                current_value = output;

                // Add the amount we got as the output of the latest intermediate trade into the trace.
                trace[pair_idx + 1].amount += current_value.amount;
            }

            if current_value.amount == 0u64.into() {
                // While the `input` to `fill_route` can be zero, the 'filling
                // loop requires that `input.amount > 0`. The only other possible
                // source of a `current_value.amount == 0` is from the saturating
                // input calculation (lifting the limiting constraint).
                // There are two reasons why no saturating can be zero:
                // + delta_1_star = ceil(delta_1), so unless delta_1 == 0, we
                // have delta_1_star > 0.
                // + delta_1 = lambda_2 * accumulated_effective_price
                // And lambda_2 != 0, since lambda_2 = r2 and r2 != 0 because
                // otherwise it would not have been indexed for that pair's
                // direction.
                // + accumulated_effective_price != 0, because for each position
                //   we have p, q != 0.
                unreachable!("current_value is nonzero")
            }

            // Now record the input we consumed and the output we gained:
            input.amount = input.amount - input_capacity;
            output.amount = output.amount + current_value.amount;
        }

        // Add the trace to the object store:
        tracing::debug!(?trace, "recording trace of filled route");
        let mut swap_execution: im::Vector<Vec<Value>> =
            self.object_get("swap_execution").unwrap_or_default();
        swap_execution.push_back(trace);
        self.object_put("swap_execution", swap_execution);

        Ok((input, output))
    }
}

impl<T: PositionManager> FillRoute for T {}
