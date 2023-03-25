use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::client::{
    stateful::create_client::CreateClientCheck,
    stateless::create_client::{client_state_is_tendermint, consensus_state_is_tendermint},
    Ics2ClientExt as _,
};

#[async_trait]
impl ActionHandler for MsgCreateClient {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        client_state_is_tendermint(self)?;
        consensus_state_is_tendermint(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.validate(self).await?;
        state.execute_create_client(self).await?;

        Ok(())
    }
}
