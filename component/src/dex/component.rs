use crate::{
    stake::{validator, View as _},
    Component, Context,
};
use penumbra_storage::{State, StateExt};
use penumbra_tct as tct;
use penumbra_transaction::{action::Undelegate, Action, Transaction};
use tendermint::abci;

pub struct Dex {
    state: State,
}

impl Dex {
    #[instrument(name = "dex", skip(state))]
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Component for Dex {
    #[instrument(name = "dex", skip(self, app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "dex", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "dex", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        // TODO: implement for Swap/SwapClaim
        Ok(())
    }

    #[instrument(name = "dex", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        // TODO: Implement for Swap/SwapClaim
        Ok(())
    }

    #[instrument(name = "dex", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
        // TODO: implement
    }

    #[instrument(name = "dex", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {
        // TODO: implement
    }
}

impl<T: StateExt> View for T {}
