use crate::{Component, Context};
use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::State;
use penumbra_transaction::{Action, Transaction};
use tendermint::abci;
use tracing::instrument;

pub struct Dex {
    _state: State,
}

impl Dex {
    #[instrument(name = "dex", skip(_state))]
    pub async fn new(_state: State) -> Self {
        Self { _state }
    }
}

#[async_trait]
impl Component for Dex {
    #[instrument(name = "dex", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "dex", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "dex", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        for action in tx.transaction_body.actions.iter() {
            match action {
                Action::PositionOpen { .. }
                | Action::PositionClose { .. }
                | Action::PositionWithdraw { .. }
                | Action::PositionRewardClaim { .. } => {
                    return Err(anyhow::anyhow!("lp actions not supported yet"));
                }
                _ => {}
            }
        }

        // TODO: implement for Swap/SwapClaim
        Ok(())
    }

    #[instrument(name = "dex", skip(self, _ctx, _tx))]
    async fn check_tx_stateful(&self, _ctx: Context, _tx: &Transaction) -> Result<()> {
        // TODO: Implement for Swap/SwapClaim
        Ok(())
    }

    #[instrument(name = "dex", skip(self, _ctx, _tx))]
    async fn execute_tx(&mut self, _ctx: Context, _tx: &Transaction) {
        // TODO: implement
    }

    #[instrument(name = "dex", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {
        // TODO: implement
    }
}
