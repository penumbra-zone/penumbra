use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::client::stateless::update_client::header_is_tendermint;
use crate::ibc::component::client::{
    stateful::update_client::UpdateClientCheck, Ics2ClientExt as _,
};

#[async_trait]
impl ActionHandler for MsgUpdateClient {
    #[instrument(name = "ibc_action", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        header_is_tendermint(self)?;

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.execute_update_client(self).await?;

        Ok(())
    }
}
