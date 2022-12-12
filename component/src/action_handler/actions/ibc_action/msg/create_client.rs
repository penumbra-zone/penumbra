use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::client::{
    stateful::create_client::CreateClientCheck,
    stateless::create_client::{client_state_is_tendermint, consensus_state_is_tendermint},
    Ics2ClientExt as _,
};

#[async_trait]
impl ActionHandler for MsgCreateClient {
    #[instrument(name = "ibc_action", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        client_state_is_tendermint(self)?;
        consensus_state_is_tendermint(self)?;

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        state.execute_create_client(self).await?;

        Ok(())
    }
}
