use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_confirm::ConnectionOpenConfirmExecute;
use crate::ibc::component::connection::stateful::connection_open_confirm::ConnectionOpenConfirmCheck;

#[async_trait]
impl ActionHandler for MsgConnectionOpenConfirm {
    #[instrument(name = "connection_open_confirm", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // NOTE: other than that the message is a well formed ConnectionOpenConfirm,
        // there is no other stateless validation to perform.

        Ok(())
    }

    #[instrument(name = "connection_open_confirm", skip(self, state, _context))]
    async fn check_stateful(&self, state: Arc<State>, _context: Arc<Transaction>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "connection_open_confirm", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        state.execute(self).await;

        Ok(())
    }
}
