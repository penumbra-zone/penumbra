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
        lp::{
            position::{self, Position},
            Reserves,
        },
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
    /// Fills a trade of a given `input` amount along a given route of `hops`,
    /// optionally using `spill_price` to put limits on execution.
    ///
    /// Note: this method will always execute at least one sub-trade along the
    /// route, even if it would exceed the spill price (i.e., the spill price is
    /// only used after consuming at least one position along the route). This
    /// covers an edge case in routing, which computes approximate spill prices:
    /// if there were two routes with very similar prices, and both of their
    /// estimated prices were underestimates, the routing could potentially
    /// switch back and forth between them without making progress. Ensuring we
    /// always consume at least one position prevents this possibility.
    ///
    /// If this behavior is not desired, use `fill_route_exact` instead.
    ///
    /// # Invariants
    ///
    /// It is an error to call `fill_route` on a route that does not have at least one position for each hop.
    #[instrument(skip(self, input, hops, spill_price))]
    async fn fill_route(
        &mut self,
        input: Value,
        hops: &[asset::Id],
        spill_price: Option<U128x128>,
    ) -> Result<(Value, Value)> {
        fill_route_inner(self, input, hops, spill_price, true).await
    }

    /// Like `fill_route`, but with exact spill price checks at the cost of
    /// potentially not making progress.
    ///
    /// Use `fill_route` instead unless you have a specific reason to use this
    /// method.
    ///
    /// # Invariants
    ///
    /// It is an error to call `fill_route_exact` on a route that does not have
    /// at least one position for each hop.
    #[instrument(skip(self, input, hops, spill_price))]
    async fn fill_route_exact(
        &mut self,
        input: Value,
        hops: &[asset::Id],
        spill_price: U128x128,
    ) -> Result<(Value, Value)> {
        fill_route_inner(self, input, hops, Some(spill_price), false).await
    }
}

impl<S: StateWrite> FillRoute for S {}

async fn fill_route_inner<S: StateWrite + Sized>(
    state: S,
    mut input: Value,
    hops: &[asset::Id],
    spill_price: Option<U128x128>,
    ensure_progress: bool,
) -> Result<(Value, Value)> {
    // Build a transaction for this execution, so if we error out at any
    // point we don't leave the state in an inconsistent state.  This is
    // particularly important for this method, because we lift position data
    // out of the state and modify it in-memory, writing it only as we fully
    // consume positions.
    let mut this = StateDelta::new(state);

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

    let mut output = Value {
        amount: 0u64.into(),
        asset_id: route
            .last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("called fill_route with empty route"))?,
    };

    let mut frontier = Frontier::load(&mut this, pairs).await?;
    tracing::debug!(?frontier, "assembled initial frontier");

    // Tracks whether we've already filled at least once, so we can skip the spill price check
    // until we've consumed at least one position.
    let mut filled_once = if ensure_progress {
        false
    } else {
        // If we don't need to ensure progress, we can act as if we've already filled once.
        true
    };

    'filling: loop {
        // INVARIANT: we must ensure that in each iteration of the loop, either:
        //
        // * we completely exhaust the input amount, or
        // * we completely exhaust the reserves of one of the active positions.

        // Phase 1 (Sensing): determine the index of the constraining position by
        // executing along the frontier, tracking which hops are
        // constraining.
        let constraining_index = frontier.sense_capacity_constraint(input);

        // Phase 2 (Filling): transactionally execute along the path, using
        // the constraint information we sensed above.
        let tx = match constraining_index {
            Some(constraining_index) => frontier.fill_constrained(constraining_index),
            None => frontier.fill_unconstrained(input),
        };

        // Phase 3 (Committing): commit the transaction if the actual price was less than the spill price.

        // WATCH OUT:
        // - `None` on the spill price means no limit.
        // - `None` on the actual price means infinite price.
        let should_apply = if let Some(spill_price) = spill_price {
            let below_limit = tx.actual_price().map(|p| p < spill_price).unwrap_or(false);

            // We should apply if we're below the limit, or we haven't yet made progress.
            below_limit || !filled_once
        } else {
            true
        };

        if !should_apply {
            tracing::debug!(
                // Hack to get an f64-formatted version of the prices; want %p but Option<_> isn't Display
                spill_price = ?spill_price.map(|x| x.to_string()),
                actual_price = ?tx.actual_price().map(|x| x.to_string()),
                "exceeded spill price, breaking loop"
            );
            // Discard the unapplied transaction, and break out of the filling loop.
            break 'filling;
        }

        let (current_input, current_output) = frontier.apply(tx);
        filled_once = true;

        // Update the input and output amounts tracked outside of the loop:
        input.amount = input.amount - current_input;
        output.amount = output.amount + current_output;

        tracing::debug!(
            ?current_input,
            ?current_output,
            input = ?input.amount,
            output = ?output.amount,
            "completed fill iteration, updating frontier"
        );

        // It's important to replace _any_ empty positions, not just the one we
        // consumed, because we might have exhausted multiple positions at once,
        // and we don't want to write empty positions into the state or process
        // them in later iterations.
        if !frontier.replace_empty_positions().await? {
            tracing::debug!("ran out of positions, breaking loop");
            break 'filling;
        }

        if constraining_index.is_none() {
            // In this case, we should have fully consumed the input amount.
            assert_eq!(input.amount, 0u64.into());
            tracing::debug!("filled input amount completely, breaking loop");
            break 'filling;
        } else {
            continue 'filling;
        }
    }

    // We need to save these positions, because we mutated their state, even
    // if we didn't fully consume their reserves.
    frontier.save();

    let trace = std::mem::take(&mut frontier.trace);
    std::mem::drop(frontier);

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
struct Frontier<S> {
    /// The list of trading pairs this frontier is for.
    pub pairs: Vec<DirectedTradingPair>,
    /// A list of the positions on the route.
    pub positions: Vec<Position>,
    /// A set of position IDs of positions contained in the frontier.
    ///
    /// This lets us correctly handle the case where we traverse the same macro-edge
    /// in opposite directions, and a position has nonzero reserves of both assets
    /// and shows up in both position streams (even though we must only use it once).
    pub position_ids: BTreeSet<position::Id>,
    /// The underlying state.
    pub state: S,
    /// A stream of positions for each pair on the route, ordered by price.
    pub positions_by_price: PositionsByPrice,
    /// A trace of the execution along the route.
    pub trace: Vec<Value>,
}

struct FrontierTx {
    new_reserves: Vec<Option<Reserves>>,
    trace: Vec<Option<Amount>>,
}

impl FrontierTx {
    fn new<S>(frontier: &Frontier<S>) -> FrontierTx {
        FrontierTx {
            new_reserves: vec![None; frontier.positions.len()],
            trace: vec![None; frontier.trace.len()],
        }
    }

    fn actual_price(&self) -> Option<U128x128> {
        let input: U128x128 = self
            .trace
            .first()
            .unwrap()
            .expect("input amount is set in a complete trace")
            .into();
        let output: U128x128 = self
            .trace
            .last()
            .unwrap()
            .expect("output amount is set in a complete trace")
            .into();

        input / output
    }
}

impl<S> std::fmt::Debug for Frontier<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frontier")
            .field("pairs", &self.pairs)
            .field("positions", &self.positions)
            .field("position_ids", &self.position_ids)
            .field("trace", &self.trace)
            .finish_non_exhaustive()
    }
}

impl<S: StateRead + StateWrite> Frontier<S> {
    async fn load(state: S, pairs: Vec<DirectedTradingPair>) -> Result<Frontier<S>> {
        let mut positions = Vec::new();
        let mut position_ids = BTreeSet::new();

        // We want to ensure that any particular position is used at most once over the route,
        // even if the route has cycles at the macro-scale. To do this, we store the streams
        // of positions for each pair, taking care to only construct one stream per distinct pair.
        let mut positions_by_price = HashMap::new();
        for pair in &pairs {
            positions_by_price
                .entry(pair.clone())
                .or_insert_with(|| state.positions_by_price(&pair));
        }

        for pair in &pairs {
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

        // Record a trace of the execution along the current route,
        // starting with all-zero amounts.
        let trace: Vec<Value> = std::iter::once(Value {
            amount: 0u64.into(),
            asset_id: pairs.first().unwrap().start,
        })
        .chain(pairs.iter().map(|pair| Value {
            amount: 0u64.into(),
            asset_id: pair.end,
        }))
        .collect();

        Ok(Frontier {
            positions,
            position_ids,
            pairs,
            state,
            positions_by_price,
            trace,
        })
    }

    fn save(&mut self) {
        for position in &self.positions {
            self.state.put_position(position.clone());
        }
    }

    /// Apply the [`FrontierTx`] to the frontier, returning the input and output
    /// amounts it added to the trace.
    fn apply(&mut self, changes: FrontierTx) -> (Amount, Amount) {
        self.trace[0].amount +=
            changes.trace[0].expect("all trace amounts must be set when applying changes");
        for (i, new_reserves) in changes.new_reserves.into_iter().enumerate() {
            let new_reserves =
                new_reserves.expect("all new reserves must be set when applying changes");
            let trace =
                changes.trace[i + 1].expect("all trace amounts must be set when applying changes");
            self.positions[i].reserves = new_reserves;
            self.trace[i + 1].amount += trace;
        }

        (
            changes.trace.first().unwrap().unwrap(),
            changes.trace.last().unwrap().unwrap(),
        )
    }

    async fn replace_empty_positions(&mut self) -> Result<bool> {
        for i in 0..self.pairs.len() {
            let desired_reserves = self.positions[i]
                .reserves_for(self.pairs[i].end)
                .expect("asset ids should match");

            // Replace any position that has been fully consumed.
            if desired_reserves == 0u64.into() {
                // If we can't find a replacement, report that failure upwards.
                if !self.replace_position(i).await? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    // Returns Ok(true) if a new position was found to replace the given one,
    // or Ok(false) if there are no more positions available for the given pair.
    #[instrument(skip(self))]
    async fn replace_position(&mut self, index: usize) -> Result<bool> {
        let replaced_position_id = self.positions[index].id();
        tracing::debug!(?replaced_position_id, "replacing position");

        // First, save the position we're about to replace.  We're going to
        // discard it, so write its updated reserves before we replace it on the
        // frontier.  The other positions will be written out either when
        // they're fully consumed, or when we finish filling.
        self.state.put_position(self.positions[index].clone());

        loop {
            let pair = &self.pairs[index];
            let next_position_id = match self
                .positions_by_price
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

            let next_position = self
                .state
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

    #[instrument(skip(self, input), fields(input = ?input.amount))]
    fn fill_unconstrained(&self, input: Value) -> FrontierTx {
        assert_eq!(input.asset_id, self.pairs.first().unwrap().start);

        let mut tx = FrontierTx::new(&self);
        // We have to manually update the trace here, because fill_forward
        // doesn't handle the input amount, only things that come after it.
        tx.trace[0] = Some(input.amount);
        // Now fill forward along the frontier, accumulating changes into the new tx.
        self.fill_forward(&mut tx, 0, input);

        tx
    }

    fn fill_constrained(&self, constraining_index: usize) -> FrontierTx {
        let mut tx = FrontierTx::new(&self);

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

        let exactly_consumed_reserves = Value {
            amount: self.positions[constraining_index]
                .reserves_for(self.pairs[constraining_index].end)
                .expect("asset ids should match"),
            asset_id: self.pairs[constraining_index].end,
        };

        tracing::debug!(
            constraining_index,
            exactly_consumed_reserves = ?exactly_consumed_reserves.amount,
            "attempting to completely consume reserves of constraining position"
        );

        // Work backwards along the path from the constraining position.
        self.fill_backward(&mut tx, constraining_index, exactly_consumed_reserves);
        // Work forwards along the path from the constraining position.
        self.fill_forward(&mut tx, constraining_index + 1, exactly_consumed_reserves);

        tx
    }

    #[instrument(skip(self, input, tx), fields(input = ?input.amount))]
    fn fill_forward(&self, tx: &mut FrontierTx, start_index: usize, input: Value) {
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

            tx.new_reserves[i] = Some(new_reserves);
            tx.trace[i + 1] = Some(output.amount);

            current_value = output;
        }
    }

    #[instrument(skip(self, output, tx), fields(output = ?output.amount))]
    fn fill_backward(&self, tx: &mut FrontierTx, start_index: usize, output: Value) {
        tracing::debug!("filling backward along frontier");
        let mut current_value = output;
        for i in (0..=start_index).rev() {
            tx.trace[i + 1] = Some(current_value.amount);

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

            tx.new_reserves[i] = Some(new_reserves);
            current_value = prev_input;
        }

        tx.trace[0] = Some(current_value.amount);
    }
}
