use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::connection::execution::connection_open_ack::ConnectionOpenAckExecute;
use crate::ibc::component::connection::stateful::connection_open_ack::ConnectionOpenAckCheck;
use crate::ibc::component::connection::stateless::connection_open_ack::{
    has_client_proof, has_client_state, has_consensus_proof,
};

#[async_trait]
impl ActionHandler for MsgConnectionOpenAck {
    #[instrument(name = "connection_open_ack", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        has_client_state(self)?;
        has_client_proof(self)?;
        has_consensus_proof(self)?;

        Ok(())
    }

    #[instrument(name = "connection_open_ack", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        state.validate(self).await?;

        Ok(())
    }

    #[instrument(name = "connection_open_ack", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        state.execute(self).await;

        Ok(())
    }
}
