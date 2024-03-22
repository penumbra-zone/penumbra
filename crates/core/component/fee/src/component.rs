pub mod rpc;
mod view;

use std::sync::Arc;

use crate::genesis;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use tendermint::abci;
use tracing::instrument;
pub use view::{StateReadExt, StateWriteExt};

// Fee component
pub struct Fee {}

#[async_trait]
impl Component for Fee {
    type AppState = genesis::Content;

    #[instrument(name = "fee", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_fee_params(genesis.fee_params.clone());
                // Put the initial gas prices
                state.put_gas_prices(genesis.fee_params.fixed_gas_prices);
            }
            None => { /* perform upgrade specific check */ }
        }
    }

    #[instrument(name = "fee", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "fee", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
        // TODO: update gas prices here eventually
    }

    #[instrument(name = "fee", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}
