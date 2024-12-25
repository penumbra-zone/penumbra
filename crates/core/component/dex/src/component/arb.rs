use std::sync::Arc;

use crate::component::{metrics, StateReadExt};
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateDelta, StateWrite};
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_proto::{DomainType as _, StateWriteProto as _};
use penumbra_sdk_sct::component::clock::EpochRead;
use tracing::instrument;

use crate::{
    component::{ExecutionCircuitBreaker, InternalDexWrite, ValueCircuitBreaker},
    event, SwapExecution,
};

use super::router::{RouteAndFill, RoutingParams};

#[async_trait]
pub trait Arbitrage: StateWrite + Sized {
    /// Attempts to extract as much as possible of the `arb_token` from the available
    /// liquidity positions, and returns the amount of `arb_token` extracted.
    #[instrument(skip(self, arb_token, routing_params))]
    async fn arbitrage(
        self: &mut Arc<Self>,
        arb_token: asset::Id,
        routing_params: RoutingParams,
    ) -> Result<Option<Value>>
    where
        Self: 'static,
    {
        tracing::debug!(?arb_token, ?routing_params, "beginning arb search");
        let arb_start = std::time::Instant::now();

        // Work in a new `StateDelta`, so we can transactionally apply any state
        // changes, and roll them back if we fail (e.g., if for some reason we
        // discover at the end that the arb wasn't profitable).
        let mut this = Arc::new(StateDelta::new(self.clone()));

        // Create a flash-loan 2^64 of the arb token to ourselves.
        let flash_loan = Value {
            asset_id: arb_token,
            amount: u64::MAX.into(),
        };

        let execution_budget = self.get_dex_params().await?.max_execution_budget;
        let execution_circuit_breaker = ExecutionCircuitBreaker::new(execution_budget);
        let Some(swap_execution) = this
            .route_and_fill(
                arb_token,
                arb_token,
                flash_loan.amount,
                routing_params,
                execution_circuit_breaker,
            )
            .await?
        else {
            return Ok(None);
        };

        let filled_input = swap_execution.input.amount;
        let output = swap_execution.output.amount;
        let unfilled_input = flash_loan
            .amount
            .checked_sub(&filled_input)
            .expect("filled input should always be <= flash loan amount");

        // Record the duration of the arb execution now, since we've computed it
        // and we might return early if it's zero-valued, but in that case we still
        // want to record how long we spent looking for it.
        metrics::histogram!(metrics::DEX_ARB_DURATION).record(arb_start.elapsed());

        // Because we're trading the arb token to itself, the total output is the
        // output from the route-and-fill, plus the unfilled input.
        let total_output = output + unfilled_input;

        // Now "repay" the flash loan by subtracting it from the total output.
        let Some(arb_profit) = total_output.checked_sub(&flash_loan.amount) else {
            // This shouldn't happen, but because route-and-fill prioritizes
            // guarantees about forward progress over precise application of
            // price limits, it technically could occur.
            tracing::debug!("mis-estimation in route-and-fill led to unprofitable arb, discarding");
            return Ok(None);
        };

        if arb_profit == 0u64.into() {
            // If we didn't make any profit, we don't need to do anything,
            // and we can just discard the state delta entirely.
            tracing::debug!("found 0-profit arb, discarding");
            return Ok(None);
        } else {
            tracing::debug!(
                ?filled_input,
                ?output,
                ?unfilled_input,
                ?total_output,
                ?arb_profit,
                "arb search detected surplus!"
            );
        }

        // TODO: this is a bit nasty, can it be simplified?
        // should this even be done "inside" the method, or all the way at the top?
        let (self2, cache) = Arc::try_unwrap(this)
            .map_err(|_| ())
            .expect("no more outstanding refs to state after routing")
            .flatten();
        std::mem::drop(self2);
        // Now there is only one reference to self again
        let mut self_mut = Arc::get_mut(self).expect("self was unique ref");
        cache.apply_to(&mut self_mut);

        // Finally, record the arb execution in the state:
        let height = self_mut.get_block_height().await?;
        let se = SwapExecution {
            traces: swap_execution.traces,
            input: Value {
                asset_id: arb_token,
                amount: filled_input,
            },
            output: Value {
                amount: filled_input + arb_profit,
                asset_id: arb_token,
            },
        };
        self_mut.set_arb_execution(height, se.clone());

        // Deduct the input surplus from the dex's VCB.
        self_mut
            .dex_vcb_debit(Value {
                amount: arb_profit,
                asset_id: arb_token,
            })
            .await?;

        // Emit an ABCI event detailing the arb execution.
        self_mut.record_proto(
            event::EventArbExecution {
                height,
                swap_execution: se,
            }
            .to_proto(),
        );
        return Ok(Some(Value {
            amount: arb_profit,
            asset_id: arb_token,
        }));
    }
}

impl<T: StateWrite> Arbitrage for T {}
