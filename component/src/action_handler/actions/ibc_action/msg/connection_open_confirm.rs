use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_confirm::ConnectionOpenConfirmExecute;
use crate::ibc::component::connection::stateful::connection_open_confirm::ConnectionOpenConfirmCheck;

#[async_trait]
impl ActionHandler for MsgConnectionOpenConfirm {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // NOTE: other than that the message is a well formed ConnectionOpenConfirm,
        // there is no other stateless validation to perform.

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.execute(self).await;

        Ok(())
    }
}
