use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
};

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateDelta, StateRead, StateWrite};
use futures::{Stream, StreamExt};
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_num::{
    fixpoint::{Error, U128x128},
    Amount,
};
use tracing::instrument;

use crate::{
    component::{metrics, PositionManager, PositionRead},
    lp::{
        position::{self, Position},
        Reserves,
    },
    DirectedTradingPair, SwapExecution, TradingPair,
};

/// An error that occurs during routing execution.
#[derive(Debug, thiserror::Error)]
pub enum FillError {
    /// Mismatch between the input asset id and the assets on either leg
    /// of the trading pair.
    #[error("input id {0:?} does not belong on pair: {1:?}")]
    AssetIdMismatch(asset::Id, TradingPair),
    /// Overflow occurred when executing against the position corresponding
    /// to the wrapped asset id.
    #[error("overflow when executing against position {0:?}")]
    ExecutionOverflow(position::Id),
    /// Route is empty or has only one hop.
    #[error("invalid route length {0} (must be at least 2)")]
    InvalidRoute(usize),
    /// Frontier position not found.
    #[error("frontier position with id {0:?}, not found")]
    MissingFrontierPosition(position::Id),
    /// Insufficient liquidity in a pair.
    #[error("insufficient liquidity in pair {0:?}")]
    InsufficientLiquidity(DirectedTradingPair),
}

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
    /// # Invariants
    ///
    /// It is an error to call `fill_route` on a route that does not have at least one position for each hop.
    ///
    /// # Errors
    /// `fill_route` can fail for a number of reasons captured by the `FillError` enum.
    ///
    /// # Panics
    /// At the moment, `fill_route` will panic on I/O failures (e.g., if the state is corrupted, or storage fails).
    #[instrument(skip(self, input, hops, spill_price))]
    async fn fill_route(
        &mut self,
        input: Value,
        hops: &[asset::Id],
        spill_price: Option<U128x128>,
    ) -> Result<SwapExecution, FillError> {
        fill_route_inner(self, input, hops, spill_price, true).await
    }
}

impl<S: StateWrite> FillRoute for S {}

async fn fill_route_inner<S: StateWrite + Sized>(
    state: S,
    mut input: Value,
    hops: &[asset::Id],
    spill_price: Option<U128x128>,
    ensure_progress: bool,
) -> Result<SwapExecution, FillError> {
    let fill_start = std::time::Instant::now();

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
            .ok_or(FillError::InvalidRoute(route.len()))?,
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
        let constraining_index = frontier.sense_capacity_constraint(input)?;

        tracing::debug!(
            ?constraining_index,
            "sensed capacity constraint, begin filling"
        );

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
            let below_limit = tx.actual_price().map(|p| p <= spill_price).unwrap_or(false);

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
        output.amount += current_output;

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
    frontier
        .save()
        .await
        .expect("writing frontier should not fail");

    // Input consists of the sum of the first value of each trace.
    let input = frontier
        .trace
        .iter()
        .map(|trace| trace.first().expect("empty trace").amount)
        .sum::<Amount>();
    // Output consists of the sum of the last value of each trace.
    let output = frontier
        .trace
        .iter()
        .map(|trace| trace.last().expect("empty trace").amount)
        .sum::<Amount>();

    let in_asset_id = frontier.pairs.first().expect("empty pairs").start;
    let out_asset_id = frontier.pairs.last().expect("empty pairs").end;

    let swap_execution = SwapExecution {
        traces: std::mem::take(&mut frontier.trace),
        input: Value {
            amount: input,
            asset_id: in_asset_id,
        },
        output: Value {
            amount: output,
            asset_id: out_asset_id,
        },
    };
    std::mem::drop(frontier);

    tracing::debug!(?swap_execution, "returning swap execution of filled route");

    // Apply the state transaction now that we've reached the end without errors.
    //
    // We have to manually extract events and push them down to the state to avoid losing them.
    // TODO: in a commit not intended to be cherry-picked, we should fix this hazardous API:
    // - rename `StateDelta::apply` to `StateDelta::apply_extracting_events`
    // - add `StateDelta::apply_with_events` that pushes the events down.
    // - go through all uses of `apply_extracting_events` and determine what behavior is correct
    let (mut state, events) = this.apply();
    for event in events {
        state.record(event);
    }

    let fill_elapsed = fill_start.elapsed();
    metrics::histogram!(metrics::DEX_ROUTE_FILL_DURATION).record(fill_elapsed);
    // cleanup / finalization
    Ok(swap_execution)
}

/// Breaksdown a route into a collection of `DirectedTradingPair`, this is mostly useful
/// for debugging right now.
fn breakdown_route(route: &[asset::Id]) -> Result<Vec<DirectedTradingPair>, FillError> {
    if route.len() < 2 {
        Err(FillError::InvalidRoute(route.len()))
    } else {
        let mut pairs = vec![];
        for pair in route.windows(2) {
            let start = pair[0];
            let end = pair[1];
            pairs.push(DirectedTradingPair::new(start, end));
        }
        Ok(pairs)
    }
}

type PositionsByPrice = BTreeMap<
    DirectedTradingPair,
    Pin<Box<dyn Stream<Item = Result<(position::Id, position::Position)>> + Send>>,
>;

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
    pub trace: Vec<Vec<Value>>,
}

struct FrontierTx {
    new_reserves: Vec<Option<Reserves>>,
    trace: Vec<Option<Amount>>,
}

impl FrontierTx {
    fn new<S>(frontier: &Frontier<S>) -> FrontierTx {
        FrontierTx {
            new_reserves: vec![None; frontier.positions.len()],
            trace: vec![None; frontier.pairs.len() + 1],
        }
    }

    fn actual_price(&self) -> Result<U128x128, Error> {
        let input: U128x128 = self
            .trace
            .first()
            .expect("input amount is set in a complete trace")
            .expect("input amount is set in a complete trace")
            .into();
        let output: U128x128 = self
            .trace
            .last()
            .expect("output amount is set in a complete trace")
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
    async fn load(state: S, pairs: Vec<DirectedTradingPair>) -> Result<Frontier<S>, FillError> {
        let mut positions = Vec::new();
        let mut position_ids = BTreeSet::new();

        // We want to ensure that any particular position is used at most once over the route,
        // even if the route has cycles at the macro-scale. To do this, we store the streams
        // of positions for each pair, taking care to only construct one stream per distinct pair.
        let mut positions_by_price = BTreeMap::new();
        for pair in &pairs {
            positions_by_price
                .entry(*pair)
                .or_insert_with(|| state.positions_by_price(pair));
        }

        for pair in &pairs {
            'next_position: loop {
                let (id, position) = positions_by_price
                    .get_mut(pair)
                    .expect("positions_by_price should have an entry for each pair")
                    .as_mut()
                    .next()
                    .await
                    .ok_or(FillError::InsufficientLiquidity(*pair))?
                    .expect("stream should not error");

                // Check that the position is not already part of the frontier.
                if !position_ids.contains(&id) {
                    position_ids.insert(id);
                    positions.push(position);

                    break 'next_position;
                }
            }
        }

        // The current trace list along the route should be initialized as empty.
        let trace: Vec<Vec<Value>> = Vec::new();

        Ok(Frontier {
            positions,
            position_ids,
            pairs,
            state,
            positions_by_price,
            trace,
        })
    }

    async fn save(&mut self) -> Result<()> {
        let context = DirectedTradingPair {
            start: self.pairs.first().expect("pairs is nonempty").start,
            end: self.pairs.last().expect("pairs is nonempty").end,
        };
        for position in &self.positions {
            self.state
                .position_execution(position.clone(), context.clone())
                .await?;
        }
        Ok(())
    }

    /// Apply the [`FrontierTx`] to the frontier, returning the input and output
    /// amounts it added to the trace.
    fn apply(&mut self, changes: FrontierTx) -> (Amount, Amount) {
        let mut trace: Vec<Value> = vec![];

        trace.push(Value {
            amount: changes.trace[0].expect("all trace amounts must be set when applying changes"),
            asset_id: self.pairs[0].start,
        });
        for (i, new_reserves) in changes.new_reserves.into_iter().enumerate() {
            let new_reserves =
                new_reserves.expect("all new reserves must be set when applying changes");
            let amount =
                changes.trace[i + 1].expect("all trace amounts must be set when applying changes");
            self.positions[i].reserves = new_reserves;
            // Pull the asset ID from the pairs.
            trace.push(Value {
                amount,
                asset_id: self.pairs[i].end,
            });
        }

        // Add the new trace
        self.trace.push(trace);

        (
            changes
                .trace
                .first()
                .expect("first should be set for a trace")
                .expect("input amount should be set for a trace"),
            changes
                .trace
                .last()
                .expect("last should be set for a trace")
                .expect("output amount should be set for a trace"),
        )
    }

    async fn replace_empty_positions(&mut self) -> Result<bool, FillError> {
        for i in 0..self.pairs.len() {
            let desired_reserves = self.positions[i]
                .reserves_for(self.pairs[i].end)
                .ok_or_else(|| {
                    FillError::AssetIdMismatch(self.pairs[i].end, self.positions[i].phi.pair)
                })?;

            // Replace any position that has been fully consumed.
            if desired_reserves == 0u64.into() {
                // If we can't find a replacement, report that failure upwards.
                if !self.replace_position(i).await {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Returns `true` if a new position was found to replace the given one,
    /// or `false`, if there are no more positions available for the given pair.
    #[instrument(skip(self))]
    async fn replace_position(&mut self, index: usize) -> bool {
        let replaced_position_id = self.positions[index].id();
        tracing::debug!(?replaced_position_id, "replacing position");

        // First, save the position we're about to replace.  We're going to
        // discard it, so write its updated reserves before we replace it on the
        // frontier.  The other positions will be written out either when
        // they're fully consumed, or when we finish filling.
        let context = DirectedTradingPair {
            start: self.pairs.first().expect("pairs is nonempty").start,
            end: self.pairs.last().expect("pairs is nonempty").end,
        };
        let updated_position = self
            .state
            .position_execution(self.positions[index].clone(), context)
            .await
            .expect("writing to storage should not fail");

        // We update the frontier cache with the updated state of the position we
        // want to discard. This protects us from cache incoherency in case we do not
        // find a suitable replacement for that position.
        self.positions[index] = updated_position;

        loop {
            let pair = &self.pairs[index];
            let (next_position_id, next_position) = match self
                .positions_by_price
                .get_mut(pair)
                .expect("positions_by_price should have an entry for each pair")
                .as_mut()
                .next()
                .await
                .transpose()
                .expect("stream doesn't error")
            {
                // If none is available, we can't keep filling, and need to signal as such.
                None => {
                    tracing::debug!(?pair, "no more positions available for pair");
                    return false;
                }
                // Otherwise, we need to check that the position is not already
                // part of the current frontier.
                Some((position_id, lp)) if !self.position_ids.contains(&position_id) => {
                    (position_id, lp)
                }
                // Otherwise, continue to the next position in the stream.
                Some(position_id) => {
                    tracing::debug!(?position_id, "skipping position already in frontier");
                    continue;
                }
            };

            tracing::debug!(
                ?next_position_id,
                ?next_position,
                "replacing constraining position in frontier",
            );

            self.position_ids.insert(next_position_id);
            self.positions[index] = next_position;

            return true;
        }
    }

    /// Senses which position along the frontier is a capacity constraint for
    /// the given input amount. If an overflow occurs during fill, report the
    /// position in an error.
    #[instrument(skip(self, input), fields(input = ?input.amount))]
    fn sense_capacity_constraint(&self, input: Value) -> Result<Option<usize>, FillError> {
        tracing::debug!(
            ?input,
            "sensing frontier capacity with trial swap input amount"
        );
        let mut constraining_index = None;
        let mut current_input = input;

        for (i, position) in self.positions.iter().enumerate() {
            if !position.phi.matches_input(current_input.asset_id) {
                tracing::error!(
                    ?current_input,
                    ?position,
                    "asset ids of input and position do not match, interrupt capacity sensing."
                );
                return Err(FillError::AssetIdMismatch(
                    current_input.asset_id,
                    position.phi.pair,
                ));
            }

            let (unfilled, new_reserves, output) = position
                .phi
                .fill(current_input, &position.reserves)
                .map_err(|_| FillError::ExecutionOverflow(position.id()))?;

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

        Ok(constraining_index)
    }

    #[instrument(skip(self, input), fields(input = ?input.amount))]
    fn fill_unconstrained(&self, input: Value) -> FrontierTx {
        assert_eq!(
            input.asset_id,
            self.pairs
                .first()
                .expect("first should be set for a trace")
                .start
        );

        let mut tx = FrontierTx::new(self);
        // We have to manually update the trace here, because fill_forward
        // doesn't handle the input amount, only things that come after it.
        tx.trace[0] = Some(input.amount);
        // Now fill forward along the frontier, accumulating changes into the new tx.
        self.fill_forward(&mut tx, 0, input);

        tx
    }

    fn fill_constrained(&self, constraining_index: usize) -> FrontierTx {
        let mut tx = FrontierTx::new(self);

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
                .expect("forward fill should not fail");

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
                .expect("backward fill should not fail")
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
