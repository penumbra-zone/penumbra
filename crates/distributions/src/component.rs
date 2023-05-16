pub mod state_key;

mod view;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_storage::StateWrite;
use tendermint::abci;
pub use view::{StateReadExt, StateWriteExt};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = genesis::AppState;

    async fn init_chain<S: StateWrite>(_state: S, _app_state: &Self::AppState) {}

    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}
