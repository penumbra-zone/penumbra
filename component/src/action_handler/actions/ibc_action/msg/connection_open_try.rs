use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_try::ConnectionOpenTryExecute;
use crate::ibc::component::connection::stateful::connection_open_try::ConnectionOpenTryCheck;
use crate::ibc::component::connection::stateless::connection_open_try::{
    has_client_proof, has_client_state, has_consensus_proof,
};

#[async_trait]
impl ActionHandler for MsgConnectionOpenTry {
    #[instrument(name = "connection_open_try", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        has_client_state(self)?;
        has_client_proof(self)?;
        has_consensus_proof(self)?;

        Ok(())
    }

    #[instrument(name = "connection_open_try", skip(self, state, _context))]
    async fn check_stateful(&self, state: Arc<State>, _context: Arc<Transaction>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "connection_open_try", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        state.execute(self).await;

        Ok(())
    }
}
