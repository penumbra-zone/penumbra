mod view;

use std::sync::Arc;

use crate::{genesis, state_key};
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

    #[instrument(name = "staking", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_fee_params(genesis.fee_params.clone());
                state.put_gas_prices(genesis.gas_prices);
            }
            None => { /* perform upgrade specific check */ }
        }
    }

    #[instrument(name = "staking", skip(state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
        let state = Arc::get_mut(state).expect("state should be unique");

        // Clear the gas prices changed marker for this block
        state.object_delete(state_key::gas_prices_changed());
    }

    #[instrument(name = "staking", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
        // TODO: update gas prices here eventually
    }

    #[instrument(name = "staking", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}
