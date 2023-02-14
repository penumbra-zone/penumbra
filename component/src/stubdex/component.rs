use std::collections::BTreeMap;

use crate::shielded_pool::{StateReadExt as _, StateWriteExt as _};
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_crypto::dex::lp::Reserves;
use penumbra_crypto::{
    asset,
    dex::{BatchSwapOutputData, TradingPair},
    SwapFlow,
};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateTransaction, StateWrite};
use tendermint::abci;
use tracing::instrument;

use super::state_key;
use super::StubCpmm;

pub struct StubDex {}

#[async_trait]
impl Component for StubDex {
    #[instrument(name = "stubdex", skip(state, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {
        // Hardcode some AMMs
        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        let penumbra = asset::REGISTRY.parse_unit("penumbra");

        state.set_stub_cpmm_reserves(
            &TradingPair::new(gm.id(), gn.id()),
            Reserves {
                r1: (10000 * 10u64.pow(gm.exponent().into())).into(),
                r2: (10000 * 10u64.pow(gn.exponent().into())).into(),
            },
        );

        state.set_stub_cpmm_reserves(
            &TradingPair::new(gm.id(), penumbra.id()),
            Reserves {
                r1: (10000 * 10u64.pow(gm.exponent().into())).into(),
                r2: (10000 * 10u64.pow(penumbra.exponent().into())).into(),
            },
        );

        state.set_stub_cpmm_reserves(
            &TradingPair::new(gn.id(), penumbra.id()),
            Reserves {
                r1: (10000 * 10u64.pow(gn.exponent().into())).into(),
                r2: (10000 * 10u64.pow(penumbra.exponent().into())).into(),
            },
        );
    }

    #[instrument(name = "stubdex", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "stubdex", skip(state, end_block))]
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock) {
        // For each batch swap during the block, calculate clearing prices and set in the JMT.
        for (trading_pair, swap_flows) in state.swap_flows() {
            let (delta_1, delta_2) = (swap_flows.0.mock_decrypt(), swap_flows.1.mock_decrypt());

            tracing::debug!(?delta_1, ?delta_2, ?trading_pair);
            let (lambda_1, lambda_2, success) =
                match state.stub_cpmm_reserves(&trading_pair).await.unwrap() {
                    Some(reserves) => {
                        tracing::debug!(?reserves, "stub cpmm is present");
                        let mut amm = StubCpmm { reserves };
                        let (lambda_1, lambda_2) = amm.trade_netted((delta_1, delta_2));
                        tracing::debug!(?lambda_1, ?lambda_2, new_reserves = ?amm.reserves);
                        state.set_stub_cpmm_reserves(&trading_pair, amm.reserves);
                        (lambda_1, lambda_2, true)
                    }
                    None => (0, 0, false),
                };

            let output_data = BatchSwapOutputData {
                height: end_block.height.try_into().unwrap(),
                trading_pair,
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                success,
            };
            tracing::debug!(?output_data);
            state.set_output_data(output_data);
        }
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

    async fn stub_cpmm_reserves(&self, trading_pair: &TradingPair) -> Result<Option<Reserves>> {
        self.get(&state_key::stub_cpmm_reserves(trading_pair)).await
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

    fn set_stub_cpmm_reserves(&mut self, trading_pair: &TradingPair, reserves: Reserves) {
        self.put(state_key::stub_cpmm_reserves(trading_pair), reserves);
    }

    fn put_swap_flow(&mut self, trading_pair: &TradingPair, swap_flow: SwapFlow) {
        // TODO: replace with IM struct later
        let mut swap_flows = self.swap_flows();
        swap_flows.insert(*trading_pair, swap_flow);
        self.object_put(state_key::swap_flows(), swap_flows)
    }
}

impl<T: StateWrite> StateWriteExt for T {}
