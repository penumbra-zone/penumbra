use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_init::ConnectionOpenInitExecute;
use crate::ibc::component::connection::stateful::connection_open_init::ConnectionOpenInitCheck;
use crate::ibc::component::connection::stateless::connection_open_init::version_is_supported;

#[async_trait]
impl ActionHandler for MsgConnectionOpenInit {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        version_is_supported(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.validate(self).await?;
        state.execute(self).await;

        Ok(())
    }
}
