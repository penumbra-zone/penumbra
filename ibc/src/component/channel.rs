use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

pub struct ICS4Channel {
    state: State,
}

#[async_trait]
impl Component for ICS4Channel {
    #[instrument(name = "ics4_channel", skip(state))]
    async fn new(state: State) -> Self {
        Self { state }
    }

    #[instrument(name = "ics4_channel", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {
    }

    #[instrument(name = "ics4_channel", skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) {
    }

    #[instrument(name = "ics4_channel", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        // Each stateless check is a distinct function in an appropriate submodule,
        // so that we can easily add new stateless checks and see a birds' eye view
        // of all of the checks we're performing.

        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                // Other IBC messages are not handled by this component.
                _ => return Ok(()),
            }
        }

        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                // Other IBC messages are not handled by this component.
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
    }

    #[instrument(name = "ics4_channel", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) {}
}

impl ICS4Channel {
}

#[async_trait]
pub trait View: StateExt {
}

impl<T: StateExt> View for T {}

