<<<<<<< HEAD
use std::collections::BTreeMap;
use std::sync::Arc;

use std::{collections::BTreeMap, sync::Arc};
=======
use std::{cell::RefCell, collections::BTreeMap, sync::Arc};
>>>>>>> bd722f1d (WIP)

use crate::{
    compactblock::view::{StateReadExt as _, StateWriteExt as _},
    dex::router::{FillRoute, PathSearch},
    Component,
};
use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_crypto::{
    dex::{BatchSwapOutputData, TradingPair},
    SwapFlow, Value,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateDelta, StateRead, StateWrite};
use tendermint::abci;
use tokio::sync::Mutex;
use tracing::instrument;

use super::state_key;

pub struct Dex {}

#[async_trait]
impl Component for Dex {
    type AppState = ();

    #[instrument(name = "dex", skip(_state, _app_state))]
    async fn init_chain<S: StateWrite>(_state: S, _app_state: &()) {}

    #[instrument(name = "dex", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "dex", skip(state, end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        // For each batch swap during the block, calculate clearing prices and set in the JMT.
        for (trading_pair, swap_flows) in state.swap_flows() {
            let (delta_1, delta_2) = (swap_flows.0.mock_decrypt(), swap_flows.1.mock_decrypt());

            tracing::debug!(?delta_1, ?delta_2, ?trading_pair);

            // Find the best route between the two assets in the trading pair.
            let (path, spill_price) = state
                // TODO: max hops should not be hardcoded
                .path_search(trading_pair.asset_1(), trading_pair.asset_2(), 4)
                .await
                .unwrap();

            tracing::debug!("path is some? {}", path.is_some());

            let (lambda_1, lambda_2, success) = if path.is_some() {
                let path = path.unwrap();
                tracing::debug!(?path);
                // path found, fill as much as we can
                // TODO: what if one of delta_1/delta_2 is zero? don't we need to fill based on the other?
                let delta_1 = Value {
                    amount: delta_1,
                    asset_id: trading_pair.asset_1(),
                };
                let delta_2 = Value {
                    amount: delta_2,
                    asset_id: trading_pair.asset_2(),
                };
                let (unfilled_1, lambda_2) = Arc::get_mut(state)
                    .expect("expected state to have no other refs")
                    .fill_route(delta_1, &path, spill_price.unwrap_or_default())
                    .await
                    .unwrap();
                let (unfilled_2, lambda_1) = Arc::get_mut(state)
                    .expect("expected state to have no other refs")
                    .fill_route(delta_2, &path, spill_price.unwrap_or_default())
                    .await
                    .unwrap();
                assert_eq!(lambda_1.asset_id, trading_pair.asset_1());
                assert_eq!(lambda_2.asset_id, trading_pair.asset_2());
                let lambda_1 = lambda_1.amount;
                let lambda_2 = lambda_2.amount;
                // TODO: don't we need to loop here to spill over and use up as much unfilled remaining assets as possible?
                tracing::debug!(?lambda_1, ?lambda_2, ?unfilled_1, ?unfilled_2);
                (lambda_1, lambda_2, true)
            } else {
                (0u64.into(), 0u64.into(), false)
            };

            let (lambda_1_1, lambda_2_2, lambda_2_1, lambda_1_2) = if success {
                (0u64.into(), 0u64.into(), lambda_2, lambda_1)
            } else {
                (delta_1, delta_2, 0u64.into(), 0u64.into())
            };

            let output_data = BatchSwapOutputData {
                height: end_block.height.try_into().unwrap(),
                trading_pair,
                delta_1,
                delta_2,
                lambda_1_1,
                lambda_2_2,
                lambda_1_2,
                lambda_2_1,
            };
            tracing::debug!(?output_data);
            Arc::get_mut(state)
                .expect("expected state to have no other refs")
                .set_output_data(output_data);
        }
    }

    #[instrument(name = "dex", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(mut _state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}

/// Extension trait providing read access to dex data.
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn output_data(
        &self,
        height: u64,
        trading_pair: TradingPair,
    ) -> Result<Option<BatchSwapOutputData>> {
        self.get(&state_key::output_data(height, trading_pair))
            .await
    }

    // Get the swap flow for the given trading pair accumulated in this block so far.
    fn swap_flow(&self, pair: &TradingPair) -> SwapFlow {
        self.swap_flows().get(pair).cloned().unwrap_or_default()
    }

    fn swap_flows(&self) -> BTreeMap<TradingPair, SwapFlow> {
        self.object_get::<BTreeMap<TradingPair, SwapFlow>>(state_key::swap_flows())
            .unwrap_or_default()
    }
}

impl<T: StateRead> StateReadExt for T {}

/// Extension trait providing write access to dex data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn set_output_data(&mut self, output_data: BatchSwapOutputData) {
        // Write the output data to the state under a known key, for querying, ...
        let height = output_data.height;
        let trading_pair = output_data.trading_pair;
        self.put(state_key::output_data(height, trading_pair), output_data);
        // ... and also add it to the compact block to be pushed out to clients.
        let mut compact_block = self.stub_compact_block();
        compact_block.swap_outputs.insert(trading_pair, output_data);
        self.stub_put_compact_block(compact_block);
    }

    fn put_swap_flow(&mut self, trading_pair: &TradingPair, swap_flow: SwapFlow) {
        // TODO: replace with IM struct later
        let mut swap_flows = self.swap_flows();
        swap_flows.insert(*trading_pair, swap_flow);
        self.object_put(state_key::swap_flows(), swap_flows)
    }
}

impl<T: StateWrite> StateWriteExt for T {}
