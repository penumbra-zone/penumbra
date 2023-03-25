use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_try::ConnectionOpenTryExecute;
use crate::ibc::component::connection::stateful::connection_open_try::ConnectionOpenTryCheck;
use crate::ibc::component::connection::stateless::connection_open_try::{
    has_client_proof, has_client_state, has_consensus_proof,
};

#[async_trait]
impl ActionHandler for MsgConnectionOpenTry {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        has_client_state(self)?;
        has_client_proof(self)?;
        has_consensus_proof(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        state.execute(self).await;

        Ok(())
    }
}
