use crate::Component;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::{StateRead, StateTransaction, StateWrite};
use tendermint::abci;
use tracing::instrument;

pub struct Dex {}

#[async_trait]
impl Component for Dex {
    #[instrument(name = "dex", skip(state, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {}

    #[instrument(name = "dex", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "dex", skip(state, end_block))]
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock) {}
}

/// Extension trait providing read access to dex data.
#[async_trait]
pub trait StateReadExt: StateRead {}

impl<T: StateRead> StateReadExt for T {}

/// Extension trait providing write access to dex data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {}

impl<T: StateWrite> StateWriteExt for T {}
