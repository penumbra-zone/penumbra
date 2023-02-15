use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_init::ConnectionOpenInitExecute;
use crate::ibc::component::connection::stateful::connection_open_init::ConnectionOpenInitCheck;
use crate::ibc::component::connection::stateless::connection_open_init::version_is_supported;

#[async_trait]
impl ActionHandler for MsgConnectionOpenInit {
    #[instrument(name = "connection_open_init", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        version_is_supported(self)?;

        Ok(())
    }

    #[instrument(name = "connection_open_init", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "connection_open_init", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.execute(self).await;

        Ok(())
    }
}
